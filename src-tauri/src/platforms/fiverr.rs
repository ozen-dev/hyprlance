use super::AuthData;

use tauri::WebviewWindow;
use tokio::time::{sleep, Duration};

pub async fn authenticate(webview: WebviewWindow) -> Result<AuthData, String> {
    // webview.eval("window.location.href = 'https://fiverr.com/'");

    sleep(Duration::from_secs(2)).await;

    Ok(AuthData {
        uid: String::from("f_1234"),
        email: String::from("f.user@email.com"),
        token: String::from("f_5678"),
    })
}
