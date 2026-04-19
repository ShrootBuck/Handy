use crate::audio_toolkit::{apply_custom_words, filter_transcription_output};
use crate::managers::model::{EngineType, ModelManager};
use crate::settings::{
    get_settings, LOCKED_APP_LANGUAGE, LOCKED_MISTRAL_TRANSCRIPTION_MODEL,
    LOCKED_SELECTED_LANGUAGE, LOCKED_WORD_CORRECTION_THRESHOLD,
};
use anyhow::Result;
use log::{debug, info};
use serde::Serialize;
use specta::Type;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Debug, Serialize)]
pub struct ModelStateEvent {
    pub event_type: String,
    pub model_id: Option<String>,
    pub model_name: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct TranscriptionManager {
    model_manager: Arc<ModelManager>,
    app_handle: AppHandle,
    current_model_id: Arc<Mutex<Option<String>>>,
}

fn encode_wav(samples: &[f32]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| anyhow::anyhow!("Failed to create WAV writer: {}", e))?;

        for sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let pcm = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(pcm)
                .map_err(|e| anyhow::anyhow!("Failed to write WAV sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| anyhow::anyhow!("Failed to finalize WAV data: {}", e))?;
    }

    Ok(cursor.into_inner())
}

impl TranscriptionManager {
    pub fn new(app_handle: &AppHandle, model_manager: Arc<ModelManager>) -> Result<Self> {
        Ok(Self {
            model_manager,
            app_handle: app_handle.clone(),
            current_model_id: Arc::new(Mutex::new(Some(
                LOCKED_MISTRAL_TRANSCRIPTION_MODEL.to_string(),
            ))),
        })
    }

    pub fn is_model_loaded(&self) -> bool {
        true
    }

    pub fn unload_model(&self) -> Result<()> {
        let mut current_model = self.current_model_id.lock().unwrap();
        *current_model = None;
        Ok(())
    }

    pub fn maybe_unload_immediately(&self, _context: &str) {}

    pub fn load_model(&self, model_id: &str) -> Result<()> {
        let model_info = self
            .model_manager
            .get_model_info(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        if !matches!(model_info.engine_type, EngineType::MistralApi) {
            return Err(anyhow::anyhow!("Only Mistral remote transcription is supported"));
        }

        let mut current_model = self.current_model_id.lock().unwrap();
        *current_model = Some(model_id.to_string());

        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "loading_completed".to_string(),
                model_id: Some(model_id.to_string()),
                model_name: Some(model_info.name),
                error: None,
            },
        );

        Ok(())
    }

    pub fn initiate_model_load(&self) {
        let _ = self.load_model(LOCKED_MISTRAL_TRANSCRIPTION_MODEL);
    }

    pub fn get_current_model(&self) -> Option<String> {
        self.current_model_id.lock().unwrap().clone()
    }

    pub fn transcribe(&self, audio: Vec<f32>) -> Result<String> {
        let st = std::time::Instant::now();
        debug!("Audio vector length: {}", audio.len());

        if audio.is_empty() {
          return Ok(String::new());
        }

        let settings = get_settings(&self.app_handle);
        let wav_bytes = encode_wav(&audio)?;
        let language = match LOCKED_SELECTED_LANGUAGE {
            "auto" => None,
            "zh-Hans" | "zh-Hant" => Some("zh"),
            other => Some(other),
        };

        let mistral_result = crate::llm_client::transcribe_with_mistral_blocking(
            &settings.mistral_transcription_base_url,
            &settings.mistral_transcription_api_key,
            LOCKED_MISTRAL_TRANSCRIPTION_MODEL,
            wav_bytes,
            language,
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        let corrected_result = if !settings.custom_words.is_empty() {
            apply_custom_words(
                &mistral_result,
                &settings.custom_words,
                LOCKED_WORD_CORRECTION_THRESHOLD,
            )
        } else {
            mistral_result
        };

        let filtered_result = filter_transcription_output(
            &corrected_result,
            LOCKED_APP_LANGUAGE,
            &settings.custom_filler_words,
        );

        info!("Transcription completed in {}ms", st.elapsed().as_millis());
        Ok(filtered_result)
    }
}

#[derive(Serialize, Clone, Debug, Type)]
pub struct GpuDeviceOption {
    pub id: i32,
    pub name: String,
    pub total_vram_mb: usize,
}

#[derive(Serialize, Clone, Debug, Type)]
pub struct AvailableAccelerators {
    pub whisper: Vec<String>,
    pub ort: Vec<String>,
    pub gpu_devices: Vec<GpuDeviceOption>,
}

pub fn apply_accelerator_settings(_app: &tauri::AppHandle) {}

pub fn get_available_accelerators() -> AvailableAccelerators {
    AvailableAccelerators {
        whisper: vec![],
        ort: vec![],
        gpu_devices: vec![],
    }
}
