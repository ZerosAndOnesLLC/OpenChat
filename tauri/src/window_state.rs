use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::WebviewWindow;

const WINDOW_STATE_FILE: &str = "window_state.json";

/// Represents the saved state of a window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// Window X position
    pub x: i32,
    /// Window Y position
    pub y: i32,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Whether the window was maximized
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1200,
            height: 800,
            maximized: false,
        }
    }
}

/// Gets the path to the window state file
fn get_state_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("OpenChat").join(WINDOW_STATE_FILE))
}

/// Saves the window state to disk
pub fn save_window_state(state: &WindowState) -> Result<(), String> {
    let path = get_state_path()
        .ok_or_else(|| "Could not determine app data directory".to_string())?;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize window state: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Failed to write window state: {}", e))?;

    log::info!("save_window_state: saved to {:?}", path);
    Ok(())
}

/// Loads the window state from disk
pub fn load_window_state() -> Option<WindowState> {
    let path = get_state_path()?;

    if !path.exists() {
        log::info!("load_window_state: no saved state found");
        return None;
    }

    let json = fs::read_to_string(&path).ok()?;
    let state: WindowState = serde_json::from_str(&json).ok()?;

    log::info!("load_window_state: loaded state {:?}", state);
    Some(state)
}

/// Captures the current window state
pub fn capture_window_state(window: &WebviewWindow) -> Option<WindowState> {
    let is_maximized = window.is_maximized().ok()?;

    // If maximized, we need to get the state before maximization
    // For now, we'll save the current position/size and the maximized flag
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;

    Some(WindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized: is_maximized,
    })
}

/// Validates that a window state is reasonable (window is at least partially visible)
/// This helps handle cases where monitors have been disconnected
pub fn validate_window_state(state: &WindowState) -> bool {
    // Basic sanity checks
    if state.width < 400 || state.height < 300 {
        return false;
    }

    // We can't easily check monitor bounds without platform-specific code,
    // but we can at least check for obviously invalid positions
    // (extremely negative values that would put the window way off-screen)
    if state.x < -10000 || state.y < -10000 {
        return false;
    }

    true
}
