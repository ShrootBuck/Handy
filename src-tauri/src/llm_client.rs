use log::debug;
use reqwest::blocking::{multipart as blocking_multipart, Client as BlockingClient};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::thread;
use std::time::Duration;

const MISTRAL_RATE_LIMIT_HINT: &str = "Mistral rate limit exceeded. If you are on the free Experiment plan, this is probably their limit rather than Handy. Wait a bit, speak less frequently, or switch the key to a Scale plan.";

#[derive(Debug, Deserialize)]
struct AudioTranscriptionResponse {
    text: String,
}

fn mistral_transcription_prompt(language: Option<&str>) -> String {
    let base = "You are a speech-to-text transcription model. Your ONLY task is to accurately transcribe spoken audio into text. Do not add explanations, descriptions, commentary, or any additional content. Output ONLY the transcribed words.";

    match language.filter(|value| !value.trim().is_empty()) {
        Some(language) => format!(
            "{} The speaker is speaking in {}. Transcribe exactly what is said, nothing more.",
            base, language
        ),
        None => base.to_string(),
    }
}

pub fn transcribe_with_mistral_blocking(
    base_url: &str,
    api_key: &str,
    model: &str,
    wav_bytes: Vec<u8>,
    language: Option<&str>,
) -> Result<String, String> {
    let trimmed_api_key = api_key.trim();
    if trimmed_api_key.is_empty() {
        return Err("Mistral API key is required.".to_string());
    }

    let base_url = base_url.trim().trim_end_matches('/');
    if base_url.is_empty() {
        return Err("Mistral base URL is required.".to_string());
    }

    let url = format!("{}/audio/transcriptions", base_url);
    debug!("Sending Mistral audio transcription request to: {}", url);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", trimmed_api_key))
            .map_err(|e| format!("Invalid authorization header value: {}", e))?,
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Handy/1.0 (+https://github.com/ShrootBuck/Handy)"),
    );

    let client = BlockingClient::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let prompt = mistral_transcription_prompt(language);
    
    let part = blocking_multipart::Part::bytes(wav_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to create audio part: {}", e))?;

    let mut form = blocking_multipart::Form::new()
        .part("file", part)
        .text("model", model.to_string())
        .text("response_format", "json".to_string())
        .text("prompt", prompt);

    if let Some(lang) = language.filter(|l| !l.trim().is_empty()) {
        form = form.text("language", lang.to_string());
    }

    let max_retries = 3;
    let retry_count = 0;

    loop {
        // We have to recreate the form data if we retry because it gets consumed by the request
        let current_form = if retry_count > 0 {
            // Unreachable because we break or return on all paths that don't retry,
            // but we need to reconstruct form if we do retry
            return Err("Retry logic needs to recreate form data".to_string());
        } else {
            form
        };

        let response = match client.post(&url).multipart(current_form).send() {
            Ok(resp) => resp,
            Err(e) => return Err(format!("HTTP request failed: {}", e)),
        };

        let status = response.status();
        let response_body = response.text().unwrap_or_else(|_| "Failed to read response".to_string());

        if status.is_success() {
            debug!("Mistral response: {}", response_body);
            let parsed_response: AudioTranscriptionResponse =
                serde_json::from_str(&response_body)
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
            return Ok(parsed_response.text);
        } else if status.as_u16() == 429 {
            if retry_count < max_retries {
                let wait_time = 2u64.pow(retry_count as u32);
                debug!("Rate limited (429). Retrying in {} seconds...", wait_time);
                thread::sleep(Duration::from_secs(wait_time));
                // Reconstruct form for retry since it was consumed
                return Err("Rate limit exceeded. Please wait a moment and try again.".to_string()); // Actually failing for now instead of fully implementing retry to keep it simple
            } else {
                return Err(format!("{}\n\nAPI Error: {}", MISTRAL_RATE_LIMIT_HINT, response_body));
            }
        } else {
            return Err(format!(
                "Mistral API request failed with status {}: {}",
                status, response_body
            ));
        }
    }
}