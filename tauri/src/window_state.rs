use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
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

/// Tracks the normal (non-maximized) window state
/// This is updated on resize/move events so we always have the correct
/// position/size to restore to, even if the window is currently maximized
static NORMAL_STATE: Mutex<Option<WindowState>> = Mutex::new(None);

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

/// Updates the tracked normal (non-maximized) window state
/// Call this on resize/move events when the window is not maximized
pub fn update_normal_state(window: &WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        return;
    }

    let position = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return,
    };
    let size = match window.inner_size() {
        Ok(s) => s,
        Err(_) => return,
    };

    let state = WindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized: false,
    };

    if let Ok(mut guard) = NORMAL_STATE.lock() {
        *guard = Some(state);
    }
}

/// Initializes the normal state tracker with the current window state
pub fn init_normal_state(window: &WebviewWindow) {
    if let Ok(mut guard) = NORMAL_STATE.lock() {
        if guard.is_none() {
            if let (Ok(position), Ok(size)) = (window.outer_position(), window.inner_size()) {
                *guard = Some(WindowState {
                    x: position.x,
                    y: position.y,
                    width: size.width,
                    height: size.height,
                    maximized: false,
                });
            }
        }
    }
}

/// Captures the current window state for saving
/// Uses the tracked normal state for position/size to handle maximized windows correctly
pub fn capture_window_state(window: &WebviewWindow) -> Option<WindowState> {
    let is_maximized = window.is_maximized().ok()?;

    if is_maximized {
        // When maximized, use the tracked normal state for position/size
        if let Ok(guard) = NORMAL_STATE.lock() {
            if let Some(ref normal) = *guard {
                return Some(WindowState {
                    x: normal.x,
                    y: normal.y,
                    width: normal.width,
                    height: normal.height,
                    maximized: true,
                });
            }
        }
        // Fallback: no tracked state, return default with maximized flag
        return Some(WindowState {
            maximized: true,
            ..WindowState::default()
        });
    }

    // Not maximized - capture current state directly
    let position = window.outer_position().ok()?;
    let size = window.inner_size().ok()?;

    Some(WindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized: false,
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
