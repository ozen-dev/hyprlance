use super::AuthData;
use std::time::Duration;
use tauri::{AppHandle, Listener, WebviewUrl, WebviewWindowBuilder};
use tokio::{sync::mpsc, time::timeout};

const EMITTER: &str = r#"
    (function() {
        const originalFetch = window.fetch;
        window.fetch = async (...args) => {
            try {
                const response = await originalFetch(...args);
                const [url, options] = args;
                if (url === "https://contra.com/api/") {
                    const clonedResponse = response.clone();
                    const responseBody = await clonedResponse.text();
                    const { data: { visitor } } = JSON.parse(responseBody);

                    if (visitor &&
                        visitor.userAccount &&
                        visitor.userAccount.id &&
                        visitor.userAccount.emailAddress &&
                        visitor.sessionId) {
                        const authData = {
                            uid: visitor.userAccount.id,
                            email: visitor.userAccount.emailAddress,
                            token: visitor.sessionId,
                        };
                        console.log("🟢 Found auth data => ", authData)
                        window.__TAURI_INTERNALS__.invoke("emit_api_data", { payload: JSON.stringify(authData) });
                    }
                }
                return response;
            } catch (error) {
                console.error("Fetch error:", error);
                return response;
            }
        };
    })();
"#;

pub async fn authenticate(app: AppHandle) -> Result<AuthData, String> {
    println!("🔐 Authenticating with contra.com...");

    let (tx, mut rx) = mpsc::channel(1);

    let listener = app.listen("hyprlance:contra-data", {
        let sender = tx.clone();
        move |event| {
            let sender = sender.clone();
            tauri::async_runtime::spawn(async move {
                match serde_json::from_str::<String>(&event.payload()) {
                    Ok(json_str) => {
                        match serde_json::from_str::<serde_json::Value>(&json_str) {
                            Ok(json_value) => {
                                if let (Some(uid), Some(email), Some(token)) = (
                                    json_value.get("uid").and_then(|v| v.as_str()),
                                    json_value.get("email").and_then(|v| v.as_str()),
                                    json_value.get("token").and_then(|v| v.as_str()),
                                ) {
                                    let auth_data = AuthData {
                                        uid: uid.to_string(),
                                        email: email.to_string(),
                                        token: token.to_string(),
                                    };
                                    println!("🟢 Successfully extracted auth data");
                                    let _ = sender.send(auth_data).await;
                                } else {
                                    eprintln!("🔴 Failed to extract required fields from inner JSON: {:?}", json_value);
                                }
                            }
                            Err(e) => {
                                eprintln!("🔴 Failed to parse inner JSON: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("🔴 Failed to parse outer JSON string: {}", e);
                    }
                }
            });
        }
    });

    let auth_window = WebviewWindowBuilder::new(
        &app,
        "auth",
        WebviewUrl::External("https://contra.com/independent/home".parse().unwrap()),
    )
    .title("Contra")
    .center()
    .inner_size(400.0, 600.0)
    .resizable(true)
    .initialization_script(EMITTER)
    .build()
    .map_err(|e| {
        eprintln!("🔐 Failed to create window: {}", e);
        e.to_string()
    })?;

    let auth_data = timeout(Duration::from_secs(180), rx.recv())
        .await
        .map_err(|_| "Authentication timed out after 3 minutes".to_string())?
        .ok_or_else(|| "Failed to receive authentication data".to_string())?;

    app.unlisten(listener);
    auth_window.close().map_err(|e| e.to_string())?;

    println!("🔐 Successfully authenticated with Contra");
    Ok(auth_data)
}
