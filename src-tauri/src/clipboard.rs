use crate::input::{self, EnigoState};
use log::info;
use tauri::{AppHandle, Manager};

pub fn paste(text: String, app_handle: AppHandle) -> Result<(), String> {
    info!("Pasting {} chars via direct typing", text.len());

    let enigo_state = app_handle
        .try_state::<EnigoState>()
        .ok_or("Enigo state not initialized")?;
    let mut enigo = enigo_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock Enigo: {}", e))?;

    input::paste_text_direct(&mut enigo, &text)
}
