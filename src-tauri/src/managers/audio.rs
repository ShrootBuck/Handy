use crate::audio_toolkit::constants::WHISPER_SAMPLE_RATE;
use crate::audio_toolkit::{list_input_devices, vad::SmoothedVad, AudioRecorder, SileroVad};
use crate::settings::get_settings;
use crate::utils;
use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::Manager;

#[derive(Clone, Debug)]
pub enum RecordingState {
    Idle,
    Recording { binding_id: String },
}

struct InnerState {
    state: RecordingState,
    recorder: Option<AudioRecorder>,
    is_open: bool,
    is_recording: bool,
}

fn create_audio_recorder(
    vad_path: &str,
    app_handle: &tauri::AppHandle,
) -> Result<AudioRecorder, anyhow::Error> {
    let silero = SileroVad::new(vad_path, 0.3)
        .map_err(|e| anyhow::anyhow!("Failed to create SileroVad: {}", e))?;
    let smoothed_vad = SmoothedVad::new(Box::new(silero), 15, 15, 2);

    let recorder = AudioRecorder::new()
        .map_err(|e| anyhow::anyhow!("Failed to create AudioRecorder: {}", e))?
        .with_vad(Box::new(smoothed_vad))
        .with_level_callback({
            let app_handle = app_handle.clone();
            move |levels| {
                utils::emit_levels(&app_handle, &levels);
            }
        });

    Ok(recorder)
}

#[derive(Clone)]
pub struct AudioRecordingManager {
    inner: Arc<Mutex<InnerState>>,
    app_handle: tauri::AppHandle,
}

impl AudioRecordingManager {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, anyhow::Error> {
        Ok(Self {
            inner: Arc::new(Mutex::new(InnerState {
                state: RecordingState::Idle,
                recorder: None,
                is_open: false,
                is_recording: false,
            })),
            app_handle: app.clone(),
        })
    }

    fn get_selected_microphone_device(&self) -> Option<cpal::Device> {
        let settings = get_settings(&self.app_handle);
        let device_name = settings.selected_microphone.as_ref()?;

        match list_input_devices() {
            Ok(devices) => devices
                .into_iter()
                .find(|d| d.name == *device_name)
                .map(|d| d.device),
            Err(e) => {
                debug!("Failed to list devices, using default: {}", e);
                None
            }
        }
    }

    fn preload_vad_inner(&self, inner: &mut InnerState) -> Result<(), anyhow::Error> {
        if inner.recorder.is_none() {
            let vad_path = self
                .app_handle
                .path()
                .resolve(
                    "resources/models/silero_vad_v4.onnx",
                    tauri::path::BaseDirectory::Resource,
                )
                .map_err(|e| anyhow::anyhow!("Failed to resolve VAD path: {}", e))?;
            let path_str = vad_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("VAD path is not valid UTF-8"))?;
            inner.recorder = Some(create_audio_recorder(path_str, &self.app_handle)?);
        }
        Ok(())
    }

    fn stop_microphone_stream_inner(inner: &mut InnerState) {
        if !inner.is_open {
            return;
        }

        if let Some(rec) = inner.recorder.as_mut() {
            if inner.is_recording {
                let _ = rec.stop();
                inner.is_recording = false;
            }
            let _ = rec.close();
        }

        inner.is_open = false;
        debug!("Microphone stream stopped");
    }

    fn start_microphone_stream_inner(&self, inner: &mut InnerState) -> Result<(), anyhow::Error> {
        if inner.is_open {
            debug!("Microphone stream already active");
            return Ok(());
        }

        let start_time = Instant::now();
        let selected_device = self.get_selected_microphone_device();

        if selected_device.is_none() {
            let has_any_device = list_input_devices()
                .map(|devices| !devices.is_empty())
                .unwrap_or(false);
            if !has_any_device {
                return Err(anyhow::anyhow!("No input device found"));
            }
        }

        self.preload_vad_inner(inner)?;

        if let Some(rec) = inner.recorder.as_mut() {
            rec.open(selected_device)
                .map_err(|e| anyhow::anyhow!("Failed to open recorder: {}", e))?;
        }

        inner.is_open = true;
        info!(
            "Microphone stream initialized in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    pub fn apply_mute(&self) {}

    pub fn remove_mute(&self) {}

    pub fn preload_vad(&self) -> Result<(), anyhow::Error> {
        let mut inner = self.inner.lock().unwrap();
        self.preload_vad_inner(&mut inner)
    }

    pub fn try_start_recording(&self, binding_id: &str) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap();

        if !matches!(inner.state, RecordingState::Idle) {
            return Err("Already recording".to_string());
        }

        if let Err(e) = self.start_microphone_stream_inner(&mut inner) {
            let msg = format!("{e}");
            error!("Failed to open microphone stream: {msg}");
            return Err(msg);
        }

        if let Some(rec) = inner.recorder.as_ref() {
            if rec.start().is_ok() {
                inner.is_recording = true;
                inner.state = RecordingState::Recording {
                    binding_id: binding_id.to_string(),
                };
                debug!("Recording started for binding {binding_id}");
                return Ok(());
            }
        }
        Err("Recorder not available".to_string())
    }

    pub fn update_selected_device(&self) -> Result<(), anyhow::Error> {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_open {
            Self::stop_microphone_stream_inner(&mut inner);
            self.start_microphone_stream_inner(&mut inner)?;
        }
        Ok(())
    }

    pub fn stop_recording(&self, binding_id: &str) -> Option<Vec<f32>> {
        let mut inner = self.inner.lock().unwrap();

        match &inner.state {
            RecordingState::Recording { binding_id: active } if active == binding_id => {
                inner.state = RecordingState::Idle;

                let samples = if let Some(rec) = inner.recorder.as_ref() {
                    match rec.stop() {
                        Ok(buf) => buf,
                        Err(e) => {
                            error!("stop() failed: {e}");
                            Vec::new()
                        }
                    }
                } else {
                    error!("Recorder not available");
                    Vec::new()
                };

                inner.is_recording = false;
                Self::stop_microphone_stream_inner(&mut inner);

                let s_len = samples.len();
                if s_len < WHISPER_SAMPLE_RATE as usize && s_len > 0 {
                    warn!(
                        "Recording is short ({} samples < {}). Padding with silence for transcription.",
                        s_len,
                        WHISPER_SAMPLE_RATE
                    );
                    let mut padded = samples;
                    padded.resize((WHISPER_SAMPLE_RATE * 5 / 4) as usize, 0.0);
                    Some(padded)
                } else {
                    Some(samples)
                }
            }
            _ => None,
        }
    }

    pub fn is_recording(&self) -> bool {
        matches!(
            self.inner.lock().unwrap().state,
            RecordingState::Recording { .. }
        )
    }

    pub fn cancel_recording(&self) {
        let mut inner = self.inner.lock().unwrap();

        if let RecordingState::Recording { .. } = inner.state {
            inner.state = RecordingState::Idle;

            if let Some(rec) = inner.recorder.as_ref() {
                let _ = rec.stop();
            }

            inner.is_recording = false;
            Self::stop_microphone_stream_inner(&mut inner);
        }
    }
}
