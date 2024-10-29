use tauri;

#[tauri::command()]
async fn get_platform_session(platform: &str) -> Result<String, String> {
    Ok(match platform {
        "contra" => format!("{}_123456789", platform),
        "fiverr" => format!("{}_123456789", platform),
        "upwork" => format!("{}_123456789", platform),
        "malt" => format!("{}_123456789", platform),
        _ => String::from("unknown_platform"),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_platform_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
