use std::sync::LazyLock;

use anyhow::{Context, Result, anyhow};
use futures_util::{StreamExt, stream};
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ChatMessage, ChatRole, ModelDescriptor,
    ProviderAuthState, ProviderAuthStatus, ProviderKind, ProviderProfile, RequestMode, TokenUsage,
};
use reqwest::{
    Client, Response,
    header::{self, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{ProviderByteStream, synthetic_chat_sse_stream};
use crate::{ProviderStreamEvent, ProviderStreamResult, openai_compatible_event_stream};

const DEFAULT_BASE_URL: &str = "https://opencode.ai/zen/v1";
static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| Client::builder().build().expect("reqwest client"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZenClientOptions {
    api_key: Option<String>,
    base_url: String,
}

impl ZenClientOptions {
    pub fn from_profile(profile: &ProviderProfile) -> Self {
        let settings = profile
            .credentials
            .clone()
            .and_then(|value| serde_json::from_value::<ZenProfileSettings>(value).ok())
            .unwrap_or_default();

        Self {
            api_key: settings.api_key,
            base_url: profile
                .base_url
                .clone()
                .or(settings.base_url)
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned()),
        }
    }

    fn persisted_credentials_with_api_key(&self, api_key: Option<String>) -> Option<Value> {
        ZenProfileSettings {
            api_key,
            base_url: (self.base_url != DEFAULT_BASE_URL).then(|| self.base_url.clone()),
        }
        .into_value()
    }
}

#[derive(Debug, Clone)]
pub struct ZenAuthStatusResult {
    pub credentials: Option<Value>,
    pub status: ProviderAuthStatus,
}

#[derive(Debug, Clone)]
pub struct ZenModelSyncResult {
    pub credentials: Option<Value>,
    pub models: Vec<ModelDescriptor>,
}

#[derive(Debug, Clone)]
pub struct ZenChatResult {
    pub completion: ChatCompletionResult,
    pub credentials: Option<Value>,
}

#[derive(Clone)]
pub struct ZenClient {
    http: Client,
    mode: ZenMode,
}

#[derive(Clone)]
enum ZenMode {
    Live(ZenClientOptions),
    Mock(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ZenProfileSettings {
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
}

impl ZenProfileSettings {
    fn into_value(self) -> Option<Value> {
        let is_empty = self.api_key.is_none() && self.base_url.is_none();
        if is_empty {
            None
        } else {
            Some(serde_json::to_value(self).expect("serialize zen credentials"))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ModelListResponse {
    data: Vec<ZenModelRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct ZenModelRecord {
    id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    model: Option<String>,
    usage: Option<ChatUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatChoice {
    finish_reason: Option<String>,
    message: ChatResponseMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
struct ApiError {
    message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ApiError {}

impl ZenClient {
    pub fn with_options(options: ZenClientOptions) -> Self {
        Self {
            http: HTTP_CLIENT.clone(),
            mode: ZenMode::Live(options),
        }
    }

    pub fn mock(response: impl Into<String>) -> Self {
        Self {
            http: HTTP_CLIENT.clone(),
            mode: ZenMode::Mock(response.into()),
        }
    }

    pub async fn auth_status(&self, profile: &ProviderProfile) -> Result<ZenAuthStatusResult> {
        match &self.mode {
            ZenMode::Mock(_) => Ok(ZenAuthStatusResult {
                credentials: profile.credentials.clone(),
                status: ProviderAuthStatus {
                    state: ProviderAuthState::Connected,
                    label: "mock@gunmetal (zen)".to_owned(),
                },
            }),
            ZenMode::Live(options) => {
                let status = if options.api_key.is_some() {
                    ProviderAuthStatus {
                        state: ProviderAuthState::Connected,
                        label: "Zen API key configured".to_owned(),
                    }
                } else {
                    ProviderAuthStatus {
                        state: ProviderAuthState::SignedOut,
                        label: "Missing Zen API key".to_owned(),
                    }
                };

                Ok(ZenAuthStatusResult {
                    credentials: options
                        .persisted_credentials_with_api_key(options.api_key.clone()),
                    status,
                })
            }
        }
    }

    pub fn clear_credentials(&self) -> Option<Value> {
        match &self.mode {
            ZenMode::Mock(_) => None,
            ZenMode::Live(options) => options.persisted_credentials_with_api_key(None),
        }
    }

    pub async fn list_models(&self, profile: &ProviderProfile) -> Result<ZenModelSyncResult> {
        match &self.mode {
            ZenMode::Mock(_) => Ok(ZenModelSyncResult {
                credentials: profile.credentials.clone(),
                models: vec![ModelDescriptor {
                    id: "zen/gpt-5.4".to_owned(),
                    provider: ProviderKind::Zen,
                    profile_id: Some(profile.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "gpt-5.4".to_owned(),
                    metadata: None,
                }],
            }),
            ZenMode::Live(options) => {
                let response = self
                    .http
                    .get(format!("{}/models", options.base_url))
                    .send()
                    .await?;

                if !response.status().is_success() {
                    return Err(read_error(response).await.into());
                }

                let payload: ModelListResponse = response.json().await?;
                let mut models = payload
                    .data
                    .into_iter()
                    .map(|model| {
                        let upstream_name = model.id;
                        ModelDescriptor {
                            id: format!("zen/{upstream_name}"),
                            provider: ProviderKind::Zen,
                            profile_id: Some(profile.id),
                            display_name: upstream_name.clone(),
                            upstream_name,
                            metadata: None,
                        }
                    })
                    .collect::<Vec<_>>();
                models.sort_by(|left, right| left.id.cmp(&right.id));

                Ok(ZenModelSyncResult {
                    credentials: options
                        .persisted_credentials_with_api_key(options.api_key.clone()),
                    models,
                })
            }
        }
    }

    pub async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ZenChatResult> {
        match &self.mode {
            ZenMode::Mock(response) => Ok(ZenChatResult {
                credentials: profile.credentials.clone(),
                completion: ChatCompletionResult {
                    model: request.model.clone(),
                    message: ChatMessage {
                        role: ChatRole::Assistant,
                        content: response.clone(),
                    },
                    finish_reason: "stop".to_owned(),
                    usage: TokenUsage {
                        input_tokens: Some(8),
                        output_tokens: Some(3),
                        total_tokens: Some(11),
                    },
                },
            }),
            ZenMode::Live(options) => {
                let api_key = Self::api_key(options)?;
                let model = request
                    .model
                    .strip_prefix("zen/")
                    .unwrap_or(&request.model)
                    .to_owned();

                let response = self
                    .http
                    .post(format!("{}/chat/completions", options.base_url))
                    .headers(self.headers(api_key)?)
                    .json(&build_zen_request_body(&model, request))
                    .send()
                    .await?;

                if !response.status().is_success() {
                    return Err(read_error(response).await.into());
                }

                let response_text = response.text().await?;
                let completion = parse_chat_completion_body(&response_text, &model)?;

                Ok(ZenChatResult {
                    credentials: None,
                    completion,
                })
            }
        }
    }

    pub async fn stream_chat_completion(
        &self,
        _profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderStreamResult> {
        match &self.mode {
            ZenMode::Mock(response) => Ok(ProviderStreamResult {
                credentials: None,
                stream: stream::iter([
                    Ok(ProviderStreamEvent::TextDelta(response.clone())),
                    Ok(ProviderStreamEvent::Complete {
                        model: request.model.clone(),
                        finish_reason: "stop".to_owned(),
                        usage: TokenUsage {
                            input_tokens: Some(8),
                            output_tokens: Some(3),
                            total_tokens: Some(11),
                        },
                    }),
                ])
                .boxed(),
            }),
            ZenMode::Live(options) => {
                let api_key = Self::api_key(options)?;
                let model = request
                    .model
                    .strip_prefix("zen/")
                    .unwrap_or(&request.model)
                    .to_owned();

                let response = self
                    .http
                    .post(format!("{}/chat/completions", options.base_url))
                    .headers(self.headers(api_key)?)
                    .json(&build_zen_request_body(&model, request))
                    .send()
                    .await?;

                if !response.status().is_success() {
                    return Err(read_error(response).await.into());
                }

                Ok(ProviderStreamResult {
                    credentials: None,
                    stream: openai_compatible_event_stream(
                        response,
                        format!("zen/{model}"),
                        |upstream_model| format!("zen/{upstream_model}"),
                    ),
                })
            }
        }
    }

    pub async fn raw_stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderByteStream> {
        match &self.mode {
            ZenMode::Mock(_) => Ok(synthetic_chat_sse_stream(
                request.model.clone(),
                self.stream_chat_completion(profile, request).await?.stream,
            )),
            ZenMode::Live(options) => {
                let api_key = Self::api_key(options)?;
                let model = request
                    .model
                    .strip_prefix("zen/")
                    .unwrap_or(&request.model)
                    .to_owned();

                let response = self
                    .http
                    .post(format!("{}/chat/completions", options.base_url))
                    .headers(self.headers(api_key)?)
                    .json(&build_zen_request_body(&model, request))
                    .send()
                    .await?;

                if !response.status().is_success() {
                    return Err(read_error(response).await.into());
                }

                Ok(response
                    .bytes_stream()
                    .map(|chunk| {
                        chunk
                            .map(|bytes| bytes.to_vec())
                            .map_err(anyhow::Error::from)
                    })
                    .boxed())
            }
        }
    }

    fn api_key(options: &ZenClientOptions) -> Result<&str> {
        options
            .api_key
            .as_deref()
            .ok_or_else(|| anyhow!("zen api key missing"))
    }

    fn headers(&self, api_key: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))?,
        );
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        Ok(headers)
    }
}

async fn read_error(response: Response) -> ApiError {
    let text = response.text().await.unwrap_or_default();
    let payload = serde_json::from_str::<Value>(&text).ok();
    let message = payload
        .as_ref()
        .and_then(|value| value.get("error"))
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .as_ref()
                .and_then(|value| value.get("message"))
                .and_then(Value::as_str)
        })
        .unwrap_or("zen request failed");

    ApiError {
        message: message.to_owned(),
    }
}

fn to_upstream_message(message: &ChatMessage) -> Value {
    json!({
        "role": match message.role {
            ChatRole::System => "system",
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        },
        "content": message.content
    })
}

fn build_zen_request_body(model: &str, request: &ChatCompletionRequest) -> Value {
    let mut body = json!({
        "model": model,
        "messages": request.messages.iter().map(to_upstream_message).collect::<Vec<_>>(),
        "stream": request.stream
    });
    let object = body.as_object_mut().expect("zen request object");

    if let Some(value) = request.options.temperature {
        object.insert("temperature".to_owned(), json!(value));
    }
    if let Some(value) = request.options.top_p {
        object.insert("top_p".to_owned(), json!(value));
    }
    if let Some(value) = request.options.max_output_tokens {
        object.insert("max_tokens".to_owned(), json!(value));
    }
    if !request.options.stop.is_empty() {
        object.insert("stop".to_owned(), json!(request.options.stop));
    }
    if !request.options.metadata.is_empty() {
        object.insert(
            "metadata".to_owned(),
            Value::Object(request.options.metadata.clone()),
        );
    }
    if matches!(request.options.mode, RequestMode::Passthrough) {
        for (key, value) in &request.options.provider_options {
            object.insert(key.clone(), value.clone());
        }
    }

    body
}

fn parse_chat_completion_body(text: &str, fallback_model: &str) -> Result<ChatCompletionResult> {
    match serde_json::from_str::<ChatCompletionResponse>(text) {
        Ok(payload) => completion_from_response(payload, fallback_model),
        Err(_error)
            if text
                .lines()
                .any(|line| line.trim_start().starts_with("data:")) =>
        {
            completion_from_sse_text(text, fallback_model)
        }
        Err(error) => {
            Err(error).with_context(|| format!("failed to decode zen chat response: {text}"))
        }
    }
}

fn completion_from_response(
    payload: ChatCompletionResponse,
    fallback_model: &str,
) -> Result<ChatCompletionResult> {
    let choice = payload
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("zen returned no choices"))?;
    let usage = usage_from_payload(payload.usage.as_ref());

    Ok(ChatCompletionResult {
        model: format!(
            "zen/{}",
            payload.model.unwrap_or_else(|| fallback_model.to_owned())
        ),
        message: ChatMessage {
            role: ChatRole::Assistant,
            content: choice.message.content.unwrap_or_default(),
        },
        finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".to_owned()),
        usage,
    })
}

fn completion_from_sse_text(text: &str, fallback_model: &str) -> Result<ChatCompletionResult> {
    let mut content = String::new();
    let mut model = fallback_model.to_owned();
    let mut finish_reason = None;
    let mut usage = TokenUsage {
        input_tokens: None,
        output_tokens: None,
        total_tokens: None,
    };

    for line in text.lines() {
        let line = line.trim();
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }

        let payload: Value = serde_json::from_str(data)
            .with_context(|| format!("failed to decode zen stream event: {data}"))?;

        if let Some(upstream_model) = payload.get("model").and_then(Value::as_str) {
            model = upstream_model.to_owned();
        }
        if let Some(event_usage) = payload
            .get("usage")
            .and_then(|value| serde_json::from_value::<ChatUsage>(value.clone()).ok())
        {
            usage = usage_from_payload(Some(&event_usage));
        }

        let Some(choice) = payload
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            continue;
        };

        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            finish_reason = Some(reason.to_owned());
        }
        if let Some(delta) = choice.get("delta") {
            if let Some(piece) = delta.get("content").and_then(Value::as_str) {
                content.push_str(piece);
            }
        } else if let Some(message) = choice.get("message")
            && let Some(piece) = message.get("content").and_then(Value::as_str)
        {
            content.push_str(piece);
        }
    }

    Ok(ChatCompletionResult {
        model: format!("zen/{model}"),
        message: ChatMessage {
            role: ChatRole::Assistant,
            content,
        },
        finish_reason: finish_reason.unwrap_or_else(|| "stop".to_owned()),
        usage,
    })
}

fn usage_from_payload(usage: Option<&ChatUsage>) -> TokenUsage {
    let input_tokens = usage.and_then(|usage| usage.prompt_tokens).map(to_u32);
    let output_tokens = usage.and_then(|usage| usage.completion_tokens).map(to_u32);
    let total_tokens = usage
        .and_then(|usage| usage.total_tokens)
        .map(to_u32)
        .or_else(|| match (input_tokens, output_tokens) {
            (Some(input), Some(output)) => Some(input.saturating_add(output)),
            _ => None,
        });

    TokenUsage {
        input_tokens,
        output_tokens,
        total_tokens,
    }
}

fn to_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use futures_util::StreamExt;
    use gunmetal_core::{
        ChatRole, ProviderAuthState, ProviderKind, ProviderProfile, RequestOptions,
    };
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, header, method, path},
    };

    use super::{ZenClient, ZenClientOptions, parse_chat_completion_body};
    use crate::ProviderStreamEvent;

    #[tokio::test]
    async fn missing_key_is_signed_out() {
        let profile = ProviderProfile {
            id: uuid::Uuid::new_v4(),
            provider: ProviderKind::Zen,
            name: "zen".to_owned(),
            base_url: None,
            enabled: true,
            credentials: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let client = ZenClient::with_options(ZenClientOptions::from_profile(&profile));

        let status = client.auth_status(&profile).await.unwrap();
        assert_eq!(status.status.state, ProviderAuthState::SignedOut);
        assert_eq!(status.status.label, "Missing Zen API key");
    }

    #[tokio::test]
    async fn lists_models_and_completes_chat() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "object": "list",
                "data": [
                    { "id": "claude-sonnet-4-5" },
                    { "id": "gpt-5.4" }
                ]
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer zen_test_key"))
            .and(body_string_contains("\"model\":\"gpt-5.4\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "model": "gpt-5.4",
                "choices": [{
                    "finish_reason": "stop",
                    "message": { "content": "GUNMETAL_ZEN_OK" }
                }],
                "usage": {
                    "prompt_tokens": 4,
                    "completion_tokens": 2,
                    "total_tokens": 6
                }
            })))
            .mount(&server)
            .await;

        let profile = ProviderProfile {
            id: uuid::Uuid::new_v4(),
            provider: ProviderKind::Zen,
            name: "zen".to_owned(),
            base_url: Some(server.uri()),
            enabled: true,
            credentials: Some(json!({
                "api_key": "zen_test_key"
            })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let client = ZenClient::with_options(ZenClientOptions::from_profile(&profile));

        let status = client.auth_status(&profile).await.unwrap();
        assert_eq!(status.status.state, ProviderAuthState::Connected);
        assert_eq!(status.status.label, "Zen API key configured");

        let models = client.list_models(&profile).await.unwrap();
        assert_eq!(models.models[0].id, "zen/claude-sonnet-4-5");
        assert_eq!(models.models[1].id, "zen/gpt-5.4");

        let completion = client
            .chat_completion(
                &profile,
                &gunmetal_core::ChatCompletionRequest {
                    model: "zen/gpt-5.4".to_owned(),
                    messages: vec![gunmetal_core::ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: false,
                    options: RequestOptions::default(),
                },
            )
            .await
            .unwrap();
        assert_eq!(completion.completion.message.content, "GUNMETAL_ZEN_OK");
        assert_eq!(completion.completion.usage.total_tokens, Some(6));
    }

    #[tokio::test]
    async fn streams_chat_chunks_without_buffering_the_full_reply() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer zen_test_key"))
            .and(body_string_contains("\"stream\":true"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(
                    concat!(
                        "data: {\"model\":\"mimo-v2-flash-free\",\"choices\":[{\"delta\":{\"content\":\"hello \"}}]}\n\n",
                        "data: {\"model\":\"mimo-v2-flash-free\",\"choices\":[{\"delta\":{\"content\":\"world\"}}]}\n\n",
                        "data: {\"model\":\"mimo-v2-flash-free\",\"choices\":[{\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":5,\"completion_tokens\":2,\"total_tokens\":7}}\n\n",
                        "data: [DONE]\n\n"
                    ),
                    "text/event-stream",
                ),
            )
            .mount(&server)
            .await;

        let profile = ProviderProfile {
            id: uuid::Uuid::new_v4(),
            provider: ProviderKind::Zen,
            name: "zen".to_owned(),
            base_url: Some(server.uri()),
            enabled: true,
            credentials: Some(json!({ "api_key": "zen_test_key" })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let client = ZenClient::with_options(ZenClientOptions::from_profile(&profile));
        let mut stream = client
            .stream_chat_completion(
                &profile,
                &gunmetal_core::ChatCompletionRequest {
                    model: "zen/mimo-v2-flash-free".to_owned(),
                    messages: vec![gunmetal_core::ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: true,
                    options: RequestOptions::default(),
                },
            )
            .await
            .unwrap()
            .stream;

        assert_eq!(
            stream.next().await.unwrap().unwrap(),
            ProviderStreamEvent::TextDelta("hello ".to_owned())
        );
        assert_eq!(
            stream.next().await.unwrap().unwrap(),
            ProviderStreamEvent::TextDelta("world".to_owned())
        );
        assert_eq!(
            stream.next().await.unwrap().unwrap(),
            ProviderStreamEvent::Complete {
                model: "zen/mimo-v2-flash-free".to_owned(),
                finish_reason: "stop".to_owned(),
                usage: gunmetal_core::TokenUsage {
                    input_tokens: Some(5),
                    output_tokens: Some(2),
                    total_tokens: Some(7),
                },
            }
        );
        assert!(stream.next().await.is_none());
    }

    #[test]
    fn parses_openrouter_style_sse_when_zen_returns_stream_for_chat_completion() {
        let body = concat!(
            ": OPENROUTER PROCESSING\n\n",
            "data: {\"id\":\"gen-1\",\"object\":\"chat.completion.chunk\",\"model\":\"tencent/hy3-preview-20260421:free\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"gun\",\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"gen-1\",\"object\":\"chat.completion.chunk\",\"model\":\"tencent/hy3-preview-20260421:free\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"metal zen\",\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"gen-1\",\"object\":\"chat.completion.chunk\",\"model\":\"tencent/hy3-preview-20260421:free\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" e2e ok\",\"role\":\"assistant\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":23,\"completion_tokens\":59,\"total_tokens\":82,\"cost\":0}}\n\n",
            "data: [DONE]\n\n",
            "data: {\"choices\":[],\"cost\":\"0\"}\n\n"
        );

        let completion = parse_chat_completion_body(body, "hy3-preview-free").unwrap();

        assert_eq!(completion.model, "zen/tencent/hy3-preview-20260421:free");
        assert_eq!(completion.message.content, "gunmetal zen e2e ok");
        assert_eq!(completion.finish_reason, "stop");
        assert_eq!(completion.usage.input_tokens, Some(23));
        assert_eq!(completion.usage.output_tokens, Some(59));
        assert_eq!(completion.usage.total_tokens, Some(82));
    }
}
