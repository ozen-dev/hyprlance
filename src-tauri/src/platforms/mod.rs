use serde::{Deserialize, Serialize};
use tauri::AppHandle;

pub mod contra;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthData {
    pub uid: String,
    pub email: String,
    pub token: String,
}

pub struct AuthService;

impl AuthService {
    pub async fn authenticate(app: AppHandle, platform: &str) -> Result<AuthData, String> {
        match platform {
            "contra" => contra::authenticate(app).await,
            _ => Err(format!("Platform '{}' is not supported.", platform)),
        }
    }
}
