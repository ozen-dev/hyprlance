use super::AuthData;

use tauri::WebviewWindow;
use tokio::time::{sleep, Duration};

pub async fn authenticate(webview: WebviewWindow) -> Result<AuthData, String> {
    // webview.eval("window.location.href = 'https://contra.com/log-in'");

    sleep(Duration::from_secs(2)).await;

    Ok(AuthData {
        uid: String::from("c_1234"),
        email: String::from("c.user@email.com"),
        token: String::from("c_5678"),
    })
}
