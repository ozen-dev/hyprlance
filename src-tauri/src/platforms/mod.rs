use std::sync::Arc;

use serde::Serialize;
use tauri::AppHandle;

pub mod contra;
pub mod proxy;

#[derive(Serialize)]
pub struct AuthData {
    pub uid: String,
    pub email: String,
    pub token: String,
}

pub struct AuthService;

impl AuthService {
    pub async fn authenticate(app: AppHandle, platform: &str) -> Result<AuthData, String> {
        let proxy_server = proxy::create_proxy_server(app.clone())
            .await
            .map_err(|e| format!("Failed to create proxy server: {}", e))?;

        let result = match platform {
            "contra" => contra::authenticate(app, Arc::clone(&proxy_server)).await,
            _ => Err(format!("Platform '{}' is not supported.", platform)),
        };

        if let Err(e) = proxy_server.shutdown() {
            eprintln!("Failed to shutdown proxy server: {}", e);
        }

        result
    }
}
