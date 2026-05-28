use enigo::{Enigo, Keyboard, Mouse, Settings};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Wrapper for Enigo to store in Tauri's managed state.
/// Enigo is wrapped in a Mutex since it requires mutable access.
pub struct EnigoState(pub Mutex<Enigo>);

impl EnigoState {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to initialize Enigo: {}", e))?;
        Ok(Self(Mutex::new(enigo)))
    }
}

/// Get the current mouse cursor position using the managed Enigo instance.
/// Returns None if the state is not available or if getting the location fails.
pub fn get_cursor_position(app_handle: &AppHandle) -> Option<(i32, i32)> {
    let enigo_state = app_handle.try_state::<EnigoState>()?;
    let enigo = enigo_state.0.lock().ok()?;
    enigo.location().ok()
}

/// Delay between each keystroke in milliseconds.
/// Prevents apps from bugging out due to receiving keys too rapidly.
const KEY_DELAY_MS: u64 = 10;

/// Pastes text directly by typing each character individually with a delay between keystrokes.
pub fn paste_text_direct(enigo: &mut Enigo, text: &str) -> Result<(), String> {
    use std::{thread, time::Duration};

    for c in text.chars() {
        let s = c.to_string();
        enigo
            .text(&s)
            .map_err(|e| format!("Failed to send text directly: {}", e))?;
        thread::sleep(Duration::from_millis(KEY_DELAY_MS));
    }

    Ok(())
}
