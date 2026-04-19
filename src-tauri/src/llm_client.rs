use log::debug;
use reqwest::blocking::Client as BlockingClient;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

const MISTRAL_RATE_LIMIT_HINT: &str = "Mistral rate limit exceeded. If you are on the free Experiment plan, this is probably their limit rather than Handy. Wait a bit, speak less frequently, or switch the key to a Scale plan.";

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

    let audio_part = Part::bytes(wav_bytes)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to build audio upload: {}", e))?;

    let mut form = Form::new()
        .text("model", model.to_string())
        .part("file", audio_part);

    if let Some(language) = language.filter(|value| !value.trim().is_empty()) {
        form = form.text("language", language.to_string());
    }

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    let response_body = response
        .text()
        .unwrap_or_else(|_| "Failed to read response".to_string());

    if status.is_success() {
        debug!("Mistral transcription response: {}", response_body);
        let parsed: TranscriptionResponse = serde_json::from_str(&response_body)
            .map_err(|e| format!("Failed to parse response: {} - body: {}", e, response_body))?;
        Ok(parsed.text.trim().to_string())
    } else if status.as_u16() == 429 {
        Err(format!(
            "{}\n\nAPI Error: {}",
            MISTRAL_RATE_LIMIT_HINT, response_body
        ))
    } else {
        Err(format!(
            "Mistral API request failed with status {}: {}",
            status, response_body
        ))
    }
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
