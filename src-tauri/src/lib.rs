mod platforms;

use platforms::{AuthData, AuthService};

/// Handles platform authentication requests from the frontend
#[tauri::command]
async fn authenticate(platform: &str, webview: tauri::WebviewWindow) -> Result<AuthData, String> {
    AuthService::authenticate(platform, webview).await
}

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![authenticate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
