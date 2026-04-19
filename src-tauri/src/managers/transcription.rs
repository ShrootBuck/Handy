use crate::settings::{
    get_settings, LOCKED_MISTRAL_TRANSCRIPTION_BASE_URL, LOCKED_MISTRAL_TRANSCRIPTION_MODEL,
};
use anyhow::Result;
use log::{debug, info};
use serde::Serialize;
use std::io::Cursor;
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
    app_handle: AppHandle,
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
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        Ok(Self {
            app_handle: app_handle.clone(),
        })
    }

    pub fn initiate_model_load(&self) {
        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "loading_completed".to_string(),
                model_id: Some(LOCKED_MISTRAL_TRANSCRIPTION_MODEL.to_string()),
                model_name: Some("Voxtral Mini (Mistral API)".to_string()),
                error: None,
            },
        );
    }

    pub fn transcribe(&self, audio: Vec<f32>) -> Result<String> {
        let st = std::time::Instant::now();
        debug!("Audio vector length: {}", audio.len());

        if audio.is_empty() {
            return Ok(String::new());
        }

        let settings = get_settings(&self.app_handle);
        let wav_bytes = encode_wav(&audio)?;

        let result = crate::llm_client::transcribe_with_mistral_blocking(
            LOCKED_MISTRAL_TRANSCRIPTION_BASE_URL,
            &settings.mistral_transcription_api_key,
            LOCKED_MISTRAL_TRANSCRIPTION_MODEL,
            wav_bytes,
            None,
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        info!("Transcription completed in {}ms", st.elapsed().as_millis());
        Ok(result)
    }
}
