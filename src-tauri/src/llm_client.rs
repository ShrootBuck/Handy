use base64::Engine;
use log::debug;
use reqwest::blocking::Client as BlockingClient;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use serde_json::json;

const MISTRAL_RATE_LIMIT_HINT: &str = "Mistral rate limit exceeded. If you are on the free Experiment plan, this is probably their limit rather than Handy. Wait a bit, speak less frequently, or switch the key to a Scale plan.";

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

fn transcription_instruction(language: Option<&str>) -> String {
    let base = "You are a speech-to-text transcription assistant. Transcribe the provided audio into plain text. Output ONLY the words that were spoken, with no preamble, no commentary, no quotation marks, no descriptions of sounds, no language labels, no summaries. If the audio is silent or empty, output an empty string. Feel free to polish/touch-up minor errors, like adjusting punctuation or removing 'uhh's.";
    match language.filter(|value| !value.trim().is_empty()) {
        Some(lang) => format!("{} The speaker is speaking in {}.", base, lang),
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

    let url = format!("{}/chat/completions", base_url);
    debug!(
        "Sending Mistral chat completions (audio) request to: {}",
        url
    );

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
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = BlockingClient::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let audio_b64 = base64::engine::general_purpose::STANDARD.encode(&wav_bytes);
    let instruction = transcription_instruction(language);

    let body = json!({
        "model": model,
        "temperature": 0.0,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "input_audio",
                        "input_audio": {
                            "data": audio_b64,
                            "format": "wav"
                        }
                    },
                    {
                        "type": "text",
                        "text": instruction
                    }
                ]
            }
        ]
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    let response_body = response
        .text()
        .unwrap_or_else(|_| "Failed to read response".to_string());

    if status.is_success() {
        debug!("Mistral chat response: {}", response_body);
        let parsed: ChatCompletionResponse = serde_json::from_str(&response_body)
            .map_err(|e| format!("Failed to parse response: {} — body: {}", e, response_body))?;
        let text = parsed
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default()
            .trim()
            .to_string();
        Ok(text)
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
