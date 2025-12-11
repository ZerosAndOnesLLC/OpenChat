mod auth;
mod window_state;

use tauri::{Emitter, Listener, Manager, State, WebviewUrl, WebviewWindowBuilder};
use url::Url;
use window_state::{capture_window_state, save_window_state, load_window_state, validate_window_state, update_normal_state, init_normal_state};

/// Returns the effective webui URL - uses localhost:3000 in dev mode, stored URL in release
fn get_effective_webui_url(stored_url: &str) -> String {
    if cfg!(debug_assertions) {
        // Dev mode: always use localhost
        "http://localhost:3000".to_string()
    } else {
        // Release mode: use the stored/server-provided URL
        stored_url.to_string()
    }
}


/// Command to show the native login screen (called on logout from web UI)
#[tauri::command]
async fn show_login_screen(app: tauri::AppHandle) -> Result<(), String> {
    log::info!("show_login_screen: closing chat window and showing login");

    // Close the chat window if it exists
    if let Some(chat_win) = app.get_webview_window("chat") {
        let _ = chat_win.close();
    }

    // Create the login window
    let login_window = WebviewWindowBuilder::new(
        &app,
        "login",
        WebviewUrl::App("login.html".into()),
    )
    .title("OpenChat - Connect")
    .inner_size(480.0, 520.0)
    .resizable(false)
    .center()
    .build()
    .map_err(|e| format!("Failed to create login window: {}", e))?;

    login_window.show().map_err(|e| format!("Failed to show login window: {}", e))?;

    Ok(())
}

/// Command to handle successful login - redirects to the web UI
#[tauri::command]
async fn login_success(
    webui_url: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let effective_url = get_effective_webui_url(&webui_url);
    log::info!("login_success: redirecting to web UI: {} (effective: {})", webui_url, effective_url);


    // Close the login window if it exists
    if let Some(login_win) = app.get_webview_window("login") {
        let _ = login_win.close();
    }

    // Load saved window state or use defaults
    let saved_state = load_window_state();
    let use_saved = saved_state.as_ref().map(|s| validate_window_state(s)).unwrap_or(false);

    // Create the chat window with saved state or defaults
    let mut builder = WebviewWindowBuilder::new(
        &app,
        "chat",
        WebviewUrl::External(effective_url.parse().map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title("OpenChat")
    .min_inner_size(800.0, 600.0);

    if use_saved {
        let state = saved_state.as_ref().unwrap();
        builder = builder
            .inner_size(state.width as f64, state.height as f64)
            .position(state.x as f64, state.y as f64);
    } else {
        builder = builder
            .inner_size(1200.0, 800.0)
            .center();
    }

    let chat_window = builder
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;

    // Apply maximized state after window is created
    if use_saved {
        if let Some(ref state) = saved_state {
            if state.maximized {
                let _ = chat_window.maximize();
            }
        }
    }

    // Set up close handler to save window state
    setup_window_close_handler(&chat_window);

    chat_window.show().map_err(|e| format!("Failed to show window: {}", e))?;

    Ok(())
}

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
            login_success,
            show_login_screen,
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

            // Check for stored credentials to determine which window to show
            let state = app.state::<auth::AppState>();
            let has_credentials = check_stored_credentials(&state);

            if has_credentials {
                // User is logged in - get the stored API URL and load web UI
                if let Some(webui_url) = get_stored_webui_url(&state) {
                    let effective_url = get_effective_webui_url(&webui_url);
                    log::info!("Found stored credentials, loading web UI: {} (effective: {})", webui_url, effective_url);

                    // Load saved window state or use defaults
                    let saved_state = load_window_state();
                    let use_saved = saved_state.as_ref().map(|s| validate_window_state(s)).unwrap_or(false);

                    let mut builder = WebviewWindowBuilder::new(
                        app,
                        "chat",
                        WebviewUrl::External(effective_url.parse().unwrap()),
                    )
                    .title("OpenChat")
                    .min_inner_size(800.0, 600.0);

                    if use_saved {
                        let ws = saved_state.as_ref().unwrap();
                        log::info!("Restoring window state: {}x{} at ({}, {}), maximized: {}",
                            ws.width, ws.height, ws.x, ws.y, ws.maximized);
                        builder = builder
                            .inner_size(ws.width as f64, ws.height as f64)
                            .position(ws.x as f64, ws.y as f64);
                    } else {
                        log::info!("No valid saved window state, using defaults");
                        builder = builder
                            .inner_size(1200.0, 800.0)
                            .center();
                    }

                    let chat_window = builder.build()?;

                    // Apply maximized state after window is created
                    if use_saved {
                        if let Some(ref ws) = saved_state {
                            if ws.maximized {
                                let _ = chat_window.maximize();
                            }
                        }
                    }

                    // Set up close handler to save window state
                    setup_window_close_handler(&chat_window);

                    chat_window.show()?;
                } else {
                    // Has credentials but no API URL (shouldn't happen, but fallback to login)
                    log::warn!("Stored credentials missing api_url, showing login");
                    show_login_window(app)?;
                }
            } else {
                // No stored credentials - show login screen
                log::info!("No stored credentials, showing login screen");
                show_login_window(app)?;
            }

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

/// Check if we have valid stored credentials
fn check_stored_credentials(state: &State<'_, auth::AppState>) -> bool {
    if let Ok(creds_guard) = state.credentials.lock() {
        if let Some(ref creds) = *creds_guard {
            // Check if not expired
            return chrono::Utc::now() < creds.expires_at;
        }
    }

    // Also check file fallback (in-memory might be empty on fresh start)
    if let Some(creds) = auth::read_stored_credentials_sync() {
        if chrono::Utc::now() < creds.expires_at {
            return true;
        }
    }

    false
}

/// Get the stored API URL from credentials
fn get_stored_api_url(state: &State<'_, auth::AppState>) -> Option<String> {
    if let Ok(creds_guard) = state.credentials.lock() {
        if let Some(ref creds) = *creds_guard {
            if chrono::Utc::now() < creds.expires_at {
                return Some(creds.api_url.clone());
            }
        }
    }

    // Also check file fallback
    if let Some(creds) = auth::read_stored_credentials_sync() {
        if chrono::Utc::now() < creds.expires_at {
            return Some(creds.api_url);
        }
    }

    None
}

/// Get the stored Web UI URL from credentials
fn get_stored_webui_url(state: &State<'_, auth::AppState>) -> Option<String> {
    if let Ok(creds_guard) = state.credentials.lock() {
        if let Some(ref creds) = *creds_guard {
            if chrono::Utc::now() < creds.expires_at {
                return Some(creds.webui_url.clone());
            }
        }
    }

    // Also check file fallback
    if let Some(creds) = auth::read_stored_credentials_sync() {
        if chrono::Utc::now() < creds.expires_at {
            return Some(creds.webui_url);
        }
    }

    None
}

/// Show the login window
fn show_login_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let login_window = WebviewWindowBuilder::new(
        app,
        "login",
        WebviewUrl::App("login.html".into()),
    )
    .title("OpenChat - Connect")
    .inner_size(480.0, 520.0)
    .resizable(false)
    .center()
    .build()?;

    login_window.show()?;
    Ok(())
}

/// Sets up handlers to track window state changes and save on close
fn setup_window_close_handler(window: &tauri::WebviewWindow) {
    // Initialize normal state tracking
    init_normal_state(window);

    let window_clone = window.clone();
    window.on_window_event(move |event| {
        match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                // Capture and save window state before closing
                if let Some(state) = capture_window_state(&window_clone) {
                    log::info!("Saving window state: {}x{} at ({}, {}), maximized: {}",
                        state.width, state.height, state.x, state.y, state.maximized);
                    if let Err(e) = save_window_state(&state) {
                        log::error!("Failed to save window state: {}", e);
                    }
                }
            }
            tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_) => {
                // Track normal (non-maximized) state on resize/move
                update_normal_state(&window_clone);
            }
            _ => {}
        }
    });
}
