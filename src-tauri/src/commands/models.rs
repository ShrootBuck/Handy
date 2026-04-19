use crate::managers::model::{ModelInfo, ModelManager};
use crate::managers::transcription::{ModelStateEvent, TranscriptionManager};
use crate::settings::{get_settings, write_settings, LOCKED_MISTRAL_TRANSCRIPTION_MODEL};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
#[specta::specta]
pub async fn get_available_models(
    model_manager: State<'_, Arc<ModelManager>>,
) -> Result<Vec<ModelInfo>, String> {
    Ok(model_manager.get_available_models())
}

/// Shared logic for switching the active model, used by both the Tauri command
/// and the tray menu handler.
///
/// Validates the model, updates the persisted setting, and loads the model
/// unless the unload timeout is set to "Immediately" (in which case the model
/// will be loaded on-demand during the next transcription).
pub fn switch_active_model(app: &AppHandle, model_id: &str) -> Result<(), String> {
    if model_id != LOCKED_MISTRAL_TRANSCRIPTION_MODEL {
        return Err(format!(
            "Only {} is supported",
            LOCKED_MISTRAL_TRANSCRIPTION_MODEL
        ));
    }

    let settings = get_settings(app);
    let mut settings = settings;
    settings.selected_model = model_id.to_string();
    write_settings(app, settings);

    let model_info = app
        .state::<Arc<ModelManager>>()
        .get_model_info(model_id)
        .ok_or_else(|| format!("Model not found: {}", model_id))?;

    let _ = app.emit(
        "model-state-changed",
        ModelStateEvent {
            event_type: "selection_changed".to_string(),
            model_id: Some(model_id.to_string()),
            model_name: Some(model_info.name.clone()),
            error: None,
        },
    );

    log::info!("Model selection changed to {}.", model_id);
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub async fn set_active_model(
    app_handle: AppHandle,
    _model_manager: State<'_, Arc<ModelManager>>,
    _transcription_manager: State<'_, Arc<TranscriptionManager>>,
    model_id: String,
) -> Result<(), String> {
    switch_active_model(&app_handle, &model_id)
}

#[tauri::command]
#[specta::specta]
pub async fn get_current_model(app_handle: AppHandle) -> Result<String, String> {
    let settings = get_settings(&app_handle);
    Ok(settings.selected_model)
}
