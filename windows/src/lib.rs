mod auth;

use tauri::Manager;
use url::Url;

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
      auth::get_stored_token,
      auth::store_token,
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

      // Register deep link handler
      let handle = app.handle().clone();
      tauri_plugin_deep_link::register("openchat", move |request| {
        let handle = handle.clone();
        tauri::async_runtime::spawn(async move {
          if let Ok(url) = Url::parse(&request) {
            // URL format: openchat://login?payload=... or openchat://pair?code=...
            let host = url.host_str().unwrap_or("");
            match host {
              "login" => {
                // Handle openchat://login?payload=...
                if let Some(payload) = url.query_pairs().find(|(key, _)| key == "payload") {
                  let _ = handle.emit("deep-link-login", payload.1.to_string());
                }
              }
              "pair" => {
                // Handle openchat://pair?code=...
                if let Some(code) = url.query_pairs().find(|(key, _)| key == "code") {
                  let _ = handle.emit("deep-link-pair", code.1.to_string());
                }
              }
              _ => {}
            }
          }
        });
      })
      .map_err(|e| format!("Failed to register deep link handler: {}", e))?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
