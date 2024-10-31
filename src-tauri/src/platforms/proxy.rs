use serde::{Deserialize, Serialize};
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Url;
use tauri::{AppHandle, Emitter};

const BUFFER_SIZE: usize = 4096;
const TIMEOUT_SECS: u64 = 30;
const PROXY_ADDRESS: &str = "127.0.0.1:8080";
pub const PROXY_URL: &str = "http://127.0.0.1:8080";

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<String>,
    pub cookies: Vec<String>,
    pub body: Vec<String>,
}

impl HttpRequest {
    pub fn parse(request: &str) -> Self {
        let lines: Vec<&str> = request.lines().collect();
        let (method, url) = Self::parse_request_line(&lines);
        let (headers, cookies, body) = Self::parse_request_content(&lines[1..]);

        HttpRequest {
            method: method.to_string(),
            url: url.to_string(),
            headers: headers.into_iter().map(|s| s.to_string()).collect(),
            cookies: cookies.into_iter().map(|s| s.to_string()).collect(),
            body: body.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn parse_request_line<'a>(lines: &'a [&str]) -> (&'a str, &'a str) {
        if let Some(line) = lines.first() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return (parts[0], parts[1]);
            }
        }
        ("", "")
    }

    fn parse_request_content<'a>(lines: &'a [&str]) -> (Vec<&'a str>, Vec<&'a str>, Vec<&'a str>) {
        let mut headers = Vec::new();
        let mut cookies = Vec::new();
        let mut body = Vec::new();
        let mut reading_headers = true;

        for &line in lines {
            if line.is_empty() {
                reading_headers = false;
                continue;
            }

            if reading_headers {
                if line.to_lowercase().starts_with("cookie:") {
                    cookies.push(line[7..].trim());
                } else {
                    headers.push(line);
                }
            } else {
                body.push(line);
            }
        }

        (headers, cookies, body)
    }

    pub fn print(&self) {
        if !self.method.is_empty() && !self.url.is_empty() {
            println!("\n{} {}\n", self.method, self.url);
        }
        if !self.headers.is_empty() {
            println!("\nHEADERS:");
            for header in &self.headers {
                println!("{}", header);
            }
        }
        if !self.cookies.is_empty() {
            println!("\nCOOKIES:");
            for cookie in &self.cookies {
                println!("{}", cookie);
            }
        }
        if !self.body.is_empty() {
            println!("\nBODY:");
            for line in &self.body {
                println!("{}", line);
            }
        }
    }
}

pub struct ProxyServer {
    listener: TcpListener,
    running: Arc<AtomicBool>,
    connections: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
    app_handle: AppHandle,
}

impl ProxyServer {
    pub fn new(app_handle: AppHandle) -> Result<Self, Error> {
        let listener = TcpListener::bind(PROXY_ADDRESS)?;
        Ok(ProxyServer {
            listener,
            running: Arc::new(AtomicBool::new(true)),
            connections: Arc::new(Mutex::new(Vec::new())),
            app_handle,
        })
    }

    pub fn get_proxy_url(&self) -> Result<Url, String> {
        Url::parse(super::proxy::PROXY_URL).map_err(|e| format!("Invalid proxy URL: {}", e))
    }

    pub fn run(&self) -> Result<(), Error> {
        println!("🤖 Proxy server running on port 8080");

        self.listener.set_nonblocking(true)?;

        while self.running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((client_stream, _)) => {
                    let app_handle = self.app_handle.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_client(client_stream, app_handle) {
                            eprintln!("🤖 Error handling client: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => eprintln!("🤖 Error accepting connection: {}", e),
            }
        }

        println!("🤖 Proxy server shutting down");
        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), Error> {
        println!("🤖 Initiating proxy server shutdown");
        self.running.store(false, Ordering::SeqCst);

        // Wait for all connections to finish
        if let Ok(mut connections) = self.connections.lock() {
            for handle in connections.drain(..) {
                let _ = handle.join();
            }
        }

        println!("🤖 Proxy server shutdown complete");
        Ok(())
    }
}

fn setup_stream(stream: &mut TcpStream) -> Result<(), Error> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))?;
    stream.set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))?;
    Ok(())
}

fn handle_connect_request(request: &str, mut client_stream: TcpStream) -> Result<(), Error> {
    let host_port = request
        .lines()
        .next()
        .ok_or_else(|| Error::new(std::io::ErrorKind::InvalidData, "Invalid request"))?
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| Error::new(std::io::ErrorKind::InvalidData, "Invalid CONNECT request"))?;

    // println!("🤖 Connecting to: {}", host_port);

    let addr = host_port
        .to_socket_addrs()
        .map_err(|e| {
            Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid address: {}", e),
            )
        })?
        .next()
        .ok_or_else(|| Error::new(std::io::ErrorKind::InvalidData, "Could not resolve address"))?;

    match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
        Ok(mut server_stream) => {
            setup_stream(&mut server_stream)?;
            client_stream.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")?;
            client_stream.flush()?;
            setup_bidirectional_tunnel(client_stream, server_stream)
        }
        Err(e) => {
            eprintln!("🤖 Failed to connect to {}: {}", host_port, e);
            client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")?;
            Err(e)
        }
    }
}

fn setup_bidirectional_tunnel(
    mut client_stream: TcpStream,
    mut server_stream: TcpStream,
) -> Result<(), Error> {
    let mut client_stream_clone = client_stream.try_clone()?;
    let mut server_stream_clone = server_stream.try_clone()?;

    for stream in [
        &client_stream,
        &server_stream,
        &client_stream_clone,
        &server_stream_clone,
    ] {
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))?;
        stream.set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))?;
    }

    let client_to_server = thread::spawn(move || -> Result<(), Error> {
        let mut buffer = [0; BUFFER_SIZE];
        loop {
            match client_stream.read(&mut buffer) {
                Ok(0) => break Ok(()),
                Ok(n) => server_stream.write_all(&buffer[..n])?,
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => {
                    eprintln!("🤖 Error reading from client: {}", e);
                    break Err(e);
                }
            }
        }
    });

    let server_to_client = thread::spawn(move || -> Result<(), Error> {
        let mut buffer = [0; BUFFER_SIZE];
        loop {
            match server_stream_clone.read(&mut buffer) {
                Ok(0) => break Ok(()),
                Ok(n) => client_stream_clone.write_all(&buffer[..n])?,
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => {
                    eprintln!("🤖 Error reading from server: {}", e);
                    break Err(e);
                }
            }
        }
    });

    let _ = client_to_server.join().unwrap_or_else(|e| {
        eprintln!("🤖 Client to server thread error: {:?}", e);
        Err(Error::new(std::io::ErrorKind::Other, "Thread panicked"))
    });
    let _ = server_to_client.join().unwrap_or_else(|e| {
        eprintln!("🤖 Server to client thread error: {:?}", e);
        Err(Error::new(std::io::ErrorKind::Other, "Thread panicked"))
    });

    Ok(())
}

fn handle_client(mut client_stream: TcpStream, app_handle: AppHandle) -> Result<(), Error> {
    setup_stream(&mut client_stream)?;

    let mut buffer = vec![0; BUFFER_SIZE];
    let bytes_read = client_stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    let http_request = HttpRequest::parse(&request);
    // http_request.print();

    if let Ok(request_json) = serde_json::to_value(&http_request) {
        let _ = app_handle.emit("new-proxy-request", request_json);
    }

    if request.starts_with("CONNECT") {
        handle_connect_request(&request, client_stream)?;
    } else {
        client_stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")?;
    }

    Ok(())
}

pub async fn create_proxy_server(app_handle: AppHandle) -> Result<Arc<ProxyServer>, Error> {
    let server = ProxyServer::new(app_handle)?;
    let server = Arc::new(server);
    let server_clone = Arc::clone(&server);

    tokio::spawn(async move {
        if let Err(e) = server_clone.run() {
            eprintln!("🤖 Proxy server error: {}", e);
        }
    });

    Ok(server)
}
