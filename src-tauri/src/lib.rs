mod platforms;

use platforms::{AuthData, AuthService};

#[tauri::command]
async fn authenticate(app_handle: tauri::AppHandle, platform: &str) -> Result<AuthData, String> {
    AuthService::authenticate(app_handle, platform).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![authenticate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
