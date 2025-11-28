mod auth;

use tauri::{Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;

const WEB_UI_URL: &str = "https://openchat.zerosandones.us/";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(auth::AppState::default())
        .invoke_handler(tauri::generate_handler![
            auth::verify_pairing_code,
            auth::get_stored_credentials,
            auth::get_stored_token,
            auth::store_credentials,
            auth::clear_credentials,
            auth::clear_token,
            auth::validate_token,
            auth::process_deep_link_payload,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Hide the default window configured in tauri.conf.json
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.close();
            }

            // Simply load the web UI - it handles its own login
            let chat_window = WebviewWindowBuilder::new(
                app,
                "chat",
                WebviewUrl::External(WEB_UI_URL.parse().unwrap()),
            )
            .title("OpenChat")
            .inner_size(1200.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .center()
            .build()?;

            chat_window.show()?;

            // Register deep link handler
            let handle = app.handle().clone();
            app.listen("deep-link://request", move |event| {
                let payload_str = event.payload().to_string();
                let handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Ok(url) = Url::parse(&payload_str) {
                        let host = url.host_str().unwrap_or("");
                        match host {
                            "login" => {
                                if let Some(payload_param) = url.query_pairs().find(|(key, _)| key == "payload") {
                                    let _ = handle.emit("deep-link-login", payload_param.1.to_string());
                                }
                            }
                            "pair" => {
                                if let Some(code) = url.query_pairs().find(|(key, _)| key == "code") {
                                    let _ = handle.emit("deep-link-pair", code.1.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
