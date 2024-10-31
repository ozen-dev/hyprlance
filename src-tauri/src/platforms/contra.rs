use crate::platforms::proxy::HttpRequest;

use super::proxy::ProxyServer;
use super::AuthData;

use serde_json::Value;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Listener, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::oneshot;
use tokio::time::sleep;

pub async fn authenticate(
    app: AppHandle,
    proxy_server: Arc<ProxyServer>,
) -> Result<AuthData, String> {
    println!("🔐 Authenticating with contra.com...");

    let app_clone = app.clone();
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let tx_clone = tx.clone();
    let event_id = Arc::new(Mutex::new(None));
    let event_id_clone = event_id.clone();

    let auth_window = WebviewWindowBuilder::new(
        &app,
        "auth",
        WebviewUrl::External("https://contra.com/independent/home".parse().unwrap()),
    )
    .title("Contra")
    .center()
    .inner_size(400.0, 600.0)
    .resizable(true)
    .proxy_url(proxy_server.get_proxy_url()?)
    .build()
    .map_err(|e| {
        eprintln!("🔐 Failed to create window: {}", e);
        e.to_string()
    })?;
    let auth_window_clone = auth_window.clone();

    *event_id.lock().unwrap() = Some(app.listen("new-proxy-request", move |event| {
        let payload = event.payload();
        if let Ok(json_value) = serde_json::from_str::<Value>(payload) {
            if let Ok(http_request) = serde_json::from_value::<HttpRequest>(json_value) {
                println!("\n------------------------\n");
                HttpRequest::print(&http_request);
                println!("\n------------------------\n");

                // let auth_data = AuthData {
                //     uid: String::from(""),
                //     email: String::from(""),
                //     token: String::from("c_event_token"),
                // };

                // if let Some(id) = event_id_clone.lock().unwrap().take() {
                //     app_clone.unlisten(id);
                // }
                // auth_window_clone.close().unwrap();
                // if let Some(tx) = tx_clone.lock().unwrap().take() {
                //     let _ = tx.send(auth_data);
                // }
            }
        }
    }));

    tokio::select! {
        auth_data = rx => {
            auth_data.map_err(|e| e.to_string())
        }
        _ = sleep(Duration::from_secs(180)) => {
            if let Some(id) = event_id.lock().unwrap().take() {
                app.unlisten(id);
            }
            auth_window.close().unwrap();
            Err(String::from("Error authenticating with Contra: timeout"))
        }
    }
}
