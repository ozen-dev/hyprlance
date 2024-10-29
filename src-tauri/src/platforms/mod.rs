use serde::Serialize;

pub mod contra;
pub mod fiverr;

/// Represents authentication data for a platform
#[derive(Serialize)]
pub struct AuthData {
    pub uid: String,
    pub email: String,
    pub token: String,
}

/// Service for handling platform authentication
pub struct AuthService;

impl AuthService {
    /// Authenticates with the specified platform
    pub async fn authenticate(
        platform: &str,
        webview: tauri::WebviewWindow,
    ) -> Result<AuthData, String> {
        match platform {
            "contra" => contra::authenticate(webview).await,
            "fiverr" => fiverr::authenticate(webview).await,
            _ => Err(format!("Platform '{}' is not supported.", platform)),
        }
    }
}
