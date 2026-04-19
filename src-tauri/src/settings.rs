use log::{debug, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

pub const APP_VERSION: &str = "1.0.0";
pub const CONFIG_PATH: &str = "config.json";
pub const LOCKED_MISTRAL_TRANSCRIPTION_MODEL: &str = "voxtral-mini-latest";
pub const LOCKED_MISTRAL_TRANSCRIPTION_BASE_URL: &str = "https://api.mistral.ai/v1";

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_binding: String,
    pub current_binding: String,
}

#[derive(Clone, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub(crate) struct SecretString(String);

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            "".fmt(f)
        } else {
            "[REDACTED]".fmt(f)
        }
    }
}

impl std::ops::Deref for SecretString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SecretString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AppSettings {
    pub bindings: HashMap<String, ShortcutBinding>,
    pub push_to_talk: bool,
    pub audio_feedback_volume: f32,
    pub mistral_transcription_api_key: SecretString,
    pub selected_microphone: Option<String>,
    pub selected_output_device: Option<String>,
    pub autostart_enabled: bool,
}

#[derive(Serialize, Deserialize)]
struct PersistedConfig {
    version: String,
    fingerprint: String,
    settings: AppSettings,
}

fn config_path(app: &AppHandle) -> PathBuf {
    crate::portable::app_data_dir(app)
        .expect("Failed to resolve app data directory")
        .join(CONFIG_PATH)
}

fn config_fingerprint() -> String {
    let mut hasher = Sha256::new();
    hasher.update(APP_VERSION.as_bytes());
    hasher.update(b"|opinionated-config-v2|");
    format!("{:x}", hasher.finalize())
}

pub fn reset_config(app: &AppHandle) {
    let path = config_path(app);
    let _ = fs::remove_file(path);
}

pub fn get_default_settings() -> AppSettings {
    #[cfg(target_os = "windows")]
    let default_shortcut = "ctrl+space";
    #[cfg(target_os = "macos")]
    let default_shortcut = "option+space";
    #[cfg(target_os = "linux")]
    let default_shortcut = "ctrl+space";
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let default_shortcut = "alt+space";

    let mut bindings = HashMap::new();
    bindings.insert(
        "transcribe".to_string(),
        ShortcutBinding {
            id: "transcribe".to_string(),
            name: "Transcribe".to_string(),
            description: "Converts your speech into text.".to_string(),
            default_binding: default_shortcut.to_string(),
            current_binding: default_shortcut.to_string(),
        },
    );
    bindings.insert(
        "cancel".to_string(),
        ShortcutBinding {
            id: "cancel".to_string(),
            name: "Cancel".to_string(),
            description: "Cancels the current recording.".to_string(),
            default_binding: "escape".to_string(),
            current_binding: "escape".to_string(),
        },
    );

    AppSettings {
        bindings,
        push_to_talk: true,
        audio_feedback_volume: 1.0,
        mistral_transcription_api_key: SecretString(String::new()),
        selected_microphone: None,
        selected_output_device: None,
        autostart_enabled: false,
    }
}

fn sanitize_settings(mut settings: AppSettings) -> AppSettings {
    let defaults = get_default_settings();

    let mut sanitized_bindings = defaults.bindings.clone();
    for (binding_id, binding) in &mut sanitized_bindings {
        if let Some(existing_binding) = settings.bindings.get(binding_id) {
            binding.current_binding = existing_binding.current_binding.clone();
        }
    }

    settings.bindings = sanitized_bindings;
    settings
}

pub fn load_or_create_app_settings(app: &AppHandle) -> AppSettings {
    let path = config_path(app);
    let expected_fingerprint = config_fingerprint();

    if let Ok(contents) = fs::read_to_string(&path) {
        match serde_json::from_str::<PersistedConfig>(&contents) {
            Ok(config)
                if config.version == APP_VERSION && config.fingerprint == expected_fingerprint =>
            {
                let settings = sanitize_settings(config.settings);
                debug!("Loaded config: {:?}", settings);
                return settings;
            }
            Ok(_) => {
                warn!("Config version/fingerprint mismatch, deleting config file");
                reset_config(app);
            }
            Err(e) => {
                warn!("Failed to parse config file: {}, deleting config file", e);
                reset_config(app);
            }
        }
    }

    let defaults = get_default_settings();
    write_settings(app, defaults.clone());
    defaults
}

pub fn get_settings(app: &AppHandle) -> AppSettings {
    load_or_create_app_settings(app)
}

pub fn write_settings(app: &AppHandle, settings: AppSettings) {
    let path = config_path(app);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let persisted = PersistedConfig {
        version: APP_VERSION.to_string(),
        fingerprint: config_fingerprint(),
        settings: sanitize_settings(settings),
    };

    let serialized = serde_json::to_string_pretty(&persisted).expect("Failed to serialize config");
    fs::write(path, serialized).expect("Failed to write config");
}

pub fn get_bindings(app: &AppHandle) -> HashMap<String, ShortcutBinding> {
    get_settings(app).bindings
}

pub fn get_stored_binding(app: &AppHandle, id: &str) -> ShortcutBinding {
    get_bindings(app)
        .get(id)
        .cloned()
        .expect("Binding should exist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_redacts_api_keys() {
        let mut settings = get_default_settings();
        settings.mistral_transcription_api_key = SecretString("secret_mistral_key_123".to_string());

        let debug_output = format!("{:?}", settings);

        assert!(!debug_output.contains("secret_mistral_key_123"));
        assert!(debug_output.contains("[REDACTED]"));
    }
}
