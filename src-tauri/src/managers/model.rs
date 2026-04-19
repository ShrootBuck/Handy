use crate::settings::{get_settings, write_settings, LOCKED_MISTRAL_TRANSCRIPTION_MODEL};
use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum EngineType {
    MistralApi,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub url: Option<String>,
    pub sha256: Option<String>,
    pub size_mb: u64,
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub partial_size: u64,
    pub is_directory: bool,
    pub engine_type: EngineType,
    pub accuracy_score: f32,
    pub speed_score: f32,
    pub supports_translation: bool,
    pub is_recommended: bool,
    pub supported_languages: Vec<String>,
    pub supports_language_selection: bool,
    pub is_custom: bool,
    pub is_remote: bool,
}

pub struct ModelManager {
    app_handle: AppHandle,
    available_models: Mutex<HashMap<String, ModelInfo>>,
}

impl ModelManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let mut available_models = HashMap::new();

        available_models.insert(
            LOCKED_MISTRAL_TRANSCRIPTION_MODEL.to_string(),
            ModelInfo {
                id: LOCKED_MISTRAL_TRANSCRIPTION_MODEL.to_string(),
                name: "Voxtral Mini (Mistral API)".to_string(),
                description:
                    "Hosted transcription through Mistral's API. No local download required, but you must add your own API key in Settings."
                        .to_string(),
                filename: String::new(),
                url: None,
                sha256: None,
                size_mb: 0,
                is_downloaded: false,
                is_downloading: false,
                partial_size: 0,
                is_directory: false,
                engine_type: EngineType::MistralApi,
                accuracy_score: 0.95,
                speed_score: 0.80,
                supports_translation: false,
                is_recommended: true,
                supported_languages: vec!["en", "es", "fr", "pt", "hi", "de", "nl", "it"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                supports_language_selection: true,
                is_custom: false,
                is_remote: true,
            },
        );

        let manager = Self {
            app_handle: app_handle.clone(),
            available_models: Mutex::new(available_models),
        };

        manager.auto_select_model_if_needed()?;
        Ok(manager)
    }

    pub fn get_available_models(&self) -> Vec<ModelInfo> {
        self.available_models
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    pub fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        self.available_models.lock().unwrap().get(model_id).cloned()
    }

    fn auto_select_model_if_needed(&self) -> Result<()> {
        let mut settings = get_settings(&self.app_handle);
        if settings.selected_model != LOCKED_MISTRAL_TRANSCRIPTION_MODEL {
            info!(
                "Forcing selected model to {}",
                LOCKED_MISTRAL_TRANSCRIPTION_MODEL
            );
            settings.selected_model = LOCKED_MISTRAL_TRANSCRIPTION_MODEL.to_string();
            write_settings(&self.app_handle, settings);
        }
        Ok(())
    }
}
