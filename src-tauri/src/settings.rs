use log::{debug, warn};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
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
pub const LOCKED_SELECTED_LANGUAGE: &str = "auto";
pub const LOCKED_APP_LANGUAGE: &str = "en";
pub const LOCKED_WORD_CORRECTION_THRESHOLD: f64 = 0.18;
pub const LOCKED_PASTE_METHOD: PasteMethod = PasteMethod::Direct;
pub const LOCKED_CLIPBOARD_HANDLING: ClipboardHandling = ClipboardHandling::DontModify;
pub const LOCKED_AUTO_SUBMIT: bool = false;
pub const LOCKED_APPEND_TRAILING_SPACE: bool = false;
pub const LOCKED_EXTRA_RECORDING_BUFFER_MS: u64 = 0;
pub const LOCKED_LAZY_STREAM_CLOSE: bool = false;
pub const LOCKED_OVERLAY_POSITION: OverlayPosition = OverlayPosition::Bottom;
pub const LOCKED_HISTORY_LIMIT: usize = 10;
pub const LOCKED_RECORDING_RETENTION_PERIOD: RecordingRetentionPeriod = RecordingRetentionPeriod::PreserveLimit;

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LogLevelVisitor;

        impl<'de> Visitor<'de> for LogLevelVisitor {
            type Value = LogLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or integer representing log level")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<LogLevel, E> {
                match value.to_lowercase().as_str() {
                    "trace" => Ok(LogLevel::Trace),
                    "debug" => Ok(LogLevel::Debug),
                    "info" => Ok(LogLevel::Info),
                    "warn" => Ok(LogLevel::Warn),
                    "error" => Ok(LogLevel::Error),
                    _ => Err(E::unknown_variant(
                        value,
                        &["trace", "debug", "info", "warn", "error"],
                    )),
                }
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<LogLevel, E> {
                match value {
                    1 => Ok(LogLevel::Trace),
                    2 => Ok(LogLevel::Debug),
                    3 => Ok(LogLevel::Info),
                    4 => Ok(LogLevel::Warn),
                    5 => Ok(LogLevel::Error),
                    _ => Err(E::invalid_value(de::Unexpected::Unsigned(value), &"1-5")),
                }
            }
        }

        deserializer.deserialize_any(LogLevelVisitor)
    }
}

impl From<LogLevel> for tauri_plugin_log::LogLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tauri_plugin_log::LogLevel::Trace,
            LogLevel::Debug => tauri_plugin_log::LogLevel::Debug,
            LogLevel::Info => tauri_plugin_log::LogLevel::Info,
            LogLevel::Warn => tauri_plugin_log::LogLevel::Warn,
            LogLevel::Error => tauri_plugin_log::LogLevel::Error,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_binding: String,
    pub current_binding: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum OverlayPosition {
    None,
    Top,
    Bottom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PasteMethod {
    None,
    CtrlV,
    CtrlShiftV,
    Direct,
    ShiftInsert,
    ExternalScript,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardHandling {
    DontModify,
    CopyToClipboard,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum AutoSubmitKey {
    Enter,
    CtrlEnter,
    CmdEnter,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum RecordingRetentionPeriod {
    Never,
    PreserveLimit,
    Days3,
    Weeks2,
    Months1,
    Months3,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum KeyboardImplementation {
    Tauri,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
pub enum SoundTheme {
    Marimba,
}

impl SoundTheme {
    pub fn to_start_path(&self) -> String {
        "resources/marimba_start.wav".to_string()
    }

    pub fn to_stop_path(&self) -> String {
        "resources/marimba_stop.wav".to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum TypingTool {
    Auto,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum WhisperAcceleratorSetting {
    Auto,
    Cpu,
    Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum OrtAcceleratorSetting {
    Auto,
    Cpu,
    Cuda,
    DirectMl,
    Rocm,
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
    pub sound_theme: SoundTheme,
    pub start_hidden: bool,
    pub autostart_enabled: bool,
    pub mistral_transcription_base_url: String,
    pub mistral_transcription_api_key: SecretString,
    pub selected_model: String,
    pub always_on_microphone: bool,
    pub selected_microphone: Option<String>,
    pub clamshell_microphone: Option<String>,
    pub selected_output_device: Option<String>,
    pub selected_language: String,
    pub overlay_position: OverlayPosition,
    pub debug_mode: bool,
    pub log_level: LogLevel,
    pub custom_words: Vec<String>,
    pub word_correction_threshold: f64,
    pub history_limit: usize,
    pub recording_retention_period: RecordingRetentionPeriod,
    pub paste_method: PasteMethod,
    pub clipboard_handling: ClipboardHandling,
    pub auto_submit: bool,
    pub auto_submit_key: AutoSubmitKey,
    pub append_trailing_space: bool,
    pub app_language: String,
    pub experimental_enabled: bool,
    pub lazy_stream_close: bool,
    pub keyboard_implementation: KeyboardImplementation,
    pub show_tray_icon: bool,
    pub paste_delay_ms: u64,
    pub typing_tool: TypingTool,
    pub external_script_path: Option<String>,
    pub custom_filler_words: Option<Vec<String>>,
    pub whisper_accelerator: WhisperAcceleratorSetting,
    pub ort_accelerator: OrtAcceleratorSetting,
    pub whisper_gpu_device: i32,
    pub extra_recording_buffer_ms: u64,
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
    hasher.update(b"|opinionated-config-v1|");
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
        sound_theme: SoundTheme::Marimba,
        start_hidden: false,
        autostart_enabled: false,
        mistral_transcription_base_url: LOCKED_MISTRAL_TRANSCRIPTION_BASE_URL.to_string(),
        mistral_transcription_api_key: SecretString(String::new()),
        selected_model: LOCKED_MISTRAL_TRANSCRIPTION_MODEL.to_string(),
        always_on_microphone: false,
        selected_microphone: None,
        clamshell_microphone: None,
        selected_output_device: None,
        selected_language: "auto".to_string(),
        overlay_position: OverlayPosition::Bottom,
        debug_mode: false,
        log_level: LogLevel::Debug,
        custom_words: Vec::new(),
        word_correction_threshold: 0.18,
        history_limit: 10,
        recording_retention_period: RecordingRetentionPeriod::PreserveLimit,
        paste_method: PasteMethod::Direct,
        clipboard_handling: ClipboardHandling::DontModify,
        auto_submit: false,
        auto_submit_key: AutoSubmitKey::Enter,
        append_trailing_space: false,
        app_language: "en".to_string(),
        experimental_enabled: false,
        lazy_stream_close: false,
        keyboard_implementation: KeyboardImplementation::Tauri,
        show_tray_icon: true,
        paste_delay_ms: 0,
        typing_tool: TypingTool::Auto,
        external_script_path: None,
        custom_filler_words: None,
        whisper_accelerator: WhisperAcceleratorSetting::Auto,
        ort_accelerator: OrtAcceleratorSetting::Auto,
        whisper_gpu_device: -1,
        extra_recording_buffer_ms: 0,
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
    settings.sound_theme = SoundTheme::Marimba;
    settings.start_hidden = false;
    settings.autostart_enabled = false;
    settings.mistral_transcription_base_url = LOCKED_MISTRAL_TRANSCRIPTION_BASE_URL.to_string();
    settings.selected_model = LOCKED_MISTRAL_TRANSCRIPTION_MODEL.to_string();
    settings.selected_language = LOCKED_SELECTED_LANGUAGE.to_string();
    settings.overlay_position = LOCKED_OVERLAY_POSITION;
    settings.debug_mode = false;
    settings.custom_words = Vec::new();
    settings.word_correction_threshold = LOCKED_WORD_CORRECTION_THRESHOLD;
    settings.history_limit = LOCKED_HISTORY_LIMIT;
    settings.recording_retention_period = LOCKED_RECORDING_RETENTION_PERIOD;
    settings.app_language = LOCKED_APP_LANGUAGE.to_string();
    settings.experimental_enabled = false;
    settings.lazy_stream_close = LOCKED_LAZY_STREAM_CLOSE;
    settings.keyboard_implementation = KeyboardImplementation::Tauri;
    settings.show_tray_icon = true;
    settings.paste_delay_ms = 0;
    settings.paste_method = LOCKED_PASTE_METHOD;
    settings.typing_tool = TypingTool::Auto;
    settings.external_script_path = None;
    settings.clipboard_handling = LOCKED_CLIPBOARD_HANDLING;
    settings.auto_submit = LOCKED_AUTO_SUBMIT;
    settings.auto_submit_key = AutoSubmitKey::Enter;
    settings.append_trailing_space = LOCKED_APPEND_TRAILING_SPACE;
    settings.custom_filler_words = None;
    settings.whisper_accelerator = WhisperAcceleratorSetting::Auto;
    settings.ort_accelerator = OrtAcceleratorSetting::Auto;
    settings.whisper_gpu_device = -1;
    settings.extra_recording_buffer_ms = LOCKED_EXTRA_RECORDING_BUFFER_MS;
    settings
}

pub fn load_or_create_app_settings(app: &AppHandle) -> AppSettings {
    let path = config_path(app);
    let expected_fingerprint = config_fingerprint();

    if let Ok(contents) = fs::read_to_string(&path) {
        match serde_json::from_str::<PersistedConfig>(&contents) {
            Ok(config) if config.version == APP_VERSION && config.fingerprint == expected_fingerprint => {
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

pub fn get_history_limit(app: &AppHandle) -> usize {
    get_settings(app).history_limit
}

pub fn get_recording_retention_period(app: &AppHandle) -> RecordingRetentionPeriod {
    get_settings(app).recording_retention_period
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
