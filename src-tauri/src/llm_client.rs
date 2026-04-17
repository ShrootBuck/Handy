use crate::settings::PostProcessProvider;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use log::debug;
use reqwest::blocking::{multipart as blocking_multipart, Client as BlockingClient};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::thread;
use std::time::Duration;

const MISTRAL_RATE_LIMIT_HINT: &str = "Mistral rate limit exceeded. If you are on the free Experiment plan, this is probably their limit rather than Handy. Wait a bit, speak less frequently, or switch the key to a Scale plan.";

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct JsonSchema {
    name: String,
    strict: bool,
    schema: Value,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
    json_schema: JsonSchema,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AudioTranscriptionResponse {
    text: String,
}

fn build_headers(provider: &PostProcessProvider, api_key: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://github.com/cjpais/Handy"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Handy/1.0 (+https://github.com/cjpais/Handy)"),
    );
    headers.insert("X-Title", HeaderValue::from_static("Handy"));

    if !api_key.is_empty() {
        if provider.id == "anthropic" {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key header value: {}", e))?,
            );
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        } else {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| format!("Invalid authorization header value: {}", e))?,
            );
        }
    }

    Ok(headers)
}

fn create_client(provider: &PostProcessProvider, api_key: &str) -> Result<reqwest::Client, String> {
    let headers = build_headers(provider, api_key)?;
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

pub async fn send_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    prompt: String,
    reasoning_effort: Option<String>,
    reasoning: Option<ReasoningConfig>,
) -> Result<Option<String>, String> {
    send_chat_completion_with_schema(
        provider,
        api_key,
        model,
        prompt,
        None,
        None,
        reasoning_effort,
        reasoning,
    )
    .await
}

pub async fn send_chat_completion_with_schema(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    user_content: String,
    system_prompt: Option<String>,
    json_schema: Option<Value>,
    reasoning_effort: Option<String>,
    reasoning: Option<ReasoningConfig>,
) -> Result<Option<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);

    debug!("Sending chat completion request to: {}", url);

    let client = create_client(provider, &api_key)?;
    let mut messages = Vec::new();

    if let Some(system) = system_prompt {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system,
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_content,
    });

    let response_format = json_schema.map(|schema| ResponseFormat {
        format_type: "json_schema".to_string(),
        json_schema: JsonSchema {
            name: "transcription_output".to_string(),
            strict: true,
            schema,
        },
    });

    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        response_format,
        reasoning_effort,
        reasoning,
    };

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    Ok(completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone()))
}

pub async fn fetch_models(
    provider: &PostProcessProvider,
    api_key: String,
) -> Result<Vec<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/models", base_url);

    debug!("Fetching models from: {}", url);

    let client = create_client(provider, &api_key)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Model list request failed ({}): {}",
            status, error_text
        ));
    }

    let parsed: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let mut models = Vec::new();

    if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
        for entry in data {
            if let Some(id) = entry.get("id").and_then(|i| i.as_str()) {
                models.push(id.to_string());
            } else if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                models.push(name.to_string());
            }
        }
    } else if let Some(array) = parsed.as_array() {
        for entry in array {
            if let Some(model) = entry.as_str() {
                models.push(model.to_string());
            }
        }
    }

    Ok(models)
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

    let trimmed_model = model.trim();
    if trimmed_model.is_empty() {
        return Err("Mistral transcription model is required.".to_string());
    }

    let uses_chat_endpoint = trimmed_model.starts_with("voxtral-small");
    let url = if uses_chat_endpoint {
        format!("{}/chat/completions", base_url)
    } else {
        format!("{}/audio/transcriptions", base_url)
    };

    debug!("Sending blocking transcription request to: {}", url);

    let prompt = mistral_transcription_prompt(language);
    let audio_b64 = uses_chat_endpoint.then(|| BASE64_STANDARD.encode(&wav_bytes));
    let max_retries = 3;

    for attempt in 1..=max_retries {
        let response = if uses_chat_endpoint {
            let request_body = serde_json::json!({
                "model": trimmed_model,
                "messages": [{
                    "role": "user",
                    "content": [
                        {"type": "input_audio", "input_audio": audio_b64.as_ref().unwrap().clone()},
                        {"type": "text", "text": prompt.clone()}
                    ]
                }],
                "temperature": 0.0
            });

            BlockingClient::new()
                .post(&url)
                .bearer_auth(trimmed_api_key)
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .header(
                    REFERER,
                    HeaderValue::from_static("https://github.com/cjpais/Handy"),
                )
                .header(
                    USER_AGENT,
                    HeaderValue::from_static("Handy/1.0 (+https://github.com/cjpais/Handy)"),
                )
                .header("X-Title", HeaderValue::from_static("Handy"))
                .json(&request_body)
                .send()
                .map_err(|e| format!("Mistral transcription request failed: {}", e))?
        } else {
            let file_part = blocking_multipart::Part::bytes(wav_bytes.clone())
                .file_name("handy-recording.wav")
                .mime_str("audio/wav")
                .map_err(|e| format!("Failed to build WAV upload: {}", e))?;

            let mut form = blocking_multipart::Form::new()
                .text("model", trimmed_model.to_string())
                .part("file", file_part);

            if let Some(language) = language.filter(|value| !value.trim().is_empty()) {
                form = form.text("language", language.to_string());
            }

            BlockingClient::new()
                .post(&url)
                .bearer_auth(trimmed_api_key)
                .header(
                    REFERER,
                    HeaderValue::from_static("https://github.com/cjpais/Handy"),
                )
                .header(
                    USER_AGENT,
                    HeaderValue::from_static("Handy/1.0 (+https://github.com/cjpais/Handy)"),
                )
                .header("X-Title", HeaderValue::from_static("Handy"))
                .multipart(form)
                .send()
                .map_err(|e| format!("Mistral transcription request failed: {}", e))?
        };

        let status = response.status();
        if status.is_success() {
            if uses_chat_endpoint {
                let completion: ChatCompletionResponse = response
                    .json()
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                return completion
                    .choices
                    .first()
                    .and_then(|choice| choice.message.content.clone())
                    .ok_or_else(|| "No transcription content in response".to_string());
            }

            let transcription: AudioTranscriptionResponse = response
                .json()
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            return Ok(transcription.text);
        }

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse().ok())
                .unwrap_or(2_u64.saturating_pow(attempt as u32));

            debug!(
                "Rate limited, retrying after {} seconds (attempt {}/{})",
                retry_after, attempt, max_retries
            );

            if attempt == max_retries {
                let error_text = response
                    .text()
                    .unwrap_or_else(|_| "Rate limited".to_string());
                return Err(format!(
                    "{} Raw response: {}",
                    MISTRAL_RATE_LIMIT_HINT, error_text
                ));
            }

            thread::sleep(Duration::from_secs(retry_after));
            continue;
        }

        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Mistral transcription failed with status {}: {}",
            status, error_text
        ));
    }

    Err("Mistral transcription failed after exhausting retries".to_string())
}
