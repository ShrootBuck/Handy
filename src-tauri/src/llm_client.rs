use log::{debug, warn};
use reqwest::blocking::Client as BlockingClient;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::time::Duration;

const MISTRAL_RATE_LIMIT_HINT: &str = "Mistral rate limit exceeded. If you are on the free Experiment plan, this is probably their limit rather than Handy. Wait a bit, speak less frequently, or switch the key to a Scale plan.";
const MAX_RETRY_ATTEMPTS: u32 = 5;
const INITIAL_BACKOFF_MS: u64 = 1000;

#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    #[serde(default)]
    text: String,
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

    let language = language.filter(|value| !value.trim().is_empty());

    let mut last_error = String::new();

    for attempt in 0..MAX_RETRY_ATTEMPTS {
        if attempt > 0 {
            let backoff = Duration::from_millis(INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1));
            warn!(
                "Retrying Mistral transcription (attempt {}/{} after {}ms backoff)...",
                attempt + 1,
                MAX_RETRY_ATTEMPTS,
                backoff.as_millis()
            );
            std::thread::sleep(backoff);
        }

        let audio_part = Part::bytes(wav_bytes.clone())
            .file_name("recording.wav")
            .mime_str("audio/wav")
            .map_err(|e| format!("Failed to build audio upload: {}", e))?;

        let mut form = Form::new()
            .text("model", model.to_string())
            .part("file", audio_part);

        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        let response = client
            .post(&url)
            .multipart(form)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e));

        match response {
            Ok(resp) => {
                let status = resp.status();
                let response_body = resp
                    .text()
                    .unwrap_or_else(|_| "Failed to read response".to_string());

                if status.is_success() {
                    debug!("Mistral transcription response: {}", response_body);
                    let parsed: TranscriptionResponse = serde_json::from_str(&response_body)
                        .map_err(|e| {
                            format!("Failed to parse response: {} - body: {}", e, response_body)
                        })?;
                    return Ok(parsed.text.trim().to_string());
                }

                let status_code = status.as_u16();
                let is_retryable = status_code == 429
                    || status_code >= 500
                    || status_code == 408;

                let error_msg = if status_code == 429 {
                    format!(
                        "{}\n\nAPI Error: {}",
                        MISTRAL_RATE_LIMIT_HINT, response_body
                    )
                } else {
                    format!(
                        "Mistral API request failed with status {}: {}",
                        status, response_body
                    )
                };

                if is_retryable && attempt + 1 < MAX_RETRY_ATTEMPTS {
                    warn!("Transcription failed with retryable status {}: {}", status_code, error_msg);
                    last_error = error_msg;
                    continue;
                }

                return Err(error_msg);
            }
            Err(e) => {
                if attempt + 1 < MAX_RETRY_ATTEMPTS {
                    warn!("Transcription HTTP error (retryable): {}", e);
                    last_error = e;
                    continue;
                }
                return Err(e);
            }
        }
    }

    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcription_response_parses_text_field() {
        let response: TranscriptionResponse =
            serde_json::from_str(r#"{"text":"hello world"}"#).expect("response should parse");

        assert_eq!(response.text, "hello world");
    }
}
