use anyhow::{Result, anyhow};
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

const DEFAULT_BASE_URL: &str = "https://opencode.ai/zen/v1";

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
            http: Client::builder().build().expect("reqwest client"),
            mode: ZenMode::Live(options),
        }
    }

    pub fn mock(response: impl Into<String>) -> Self {
        Self {
            http: Client::builder().build().expect("reqwest client"),
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

                let payload: ChatCompletionResponse = response.json().await?;
                let choice = payload
                    .choices
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow!("zen returned no choices"))?;
                let input_tokens = payload
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.prompt_tokens)
                    .map(to_u32);
                let output_tokens = payload
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.completion_tokens)
                    .map(to_u32);
                let total_tokens = payload
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.total_tokens)
                    .map(to_u32)
                    .or_else(|| match (input_tokens, output_tokens) {
                        (Some(input), Some(output)) => Some(input.saturating_add(output)),
                        _ => None,
                    });

                Ok(ZenChatResult {
                    credentials: options
                        .persisted_credentials_with_api_key(options.api_key.clone()),
                    completion: ChatCompletionResult {
                        model: format!("zen/{}", payload.model.unwrap_or_else(|| model.to_owned())),
                        message: ChatMessage {
                            role: ChatRole::Assistant,
                            content: choice.message.content.unwrap_or_default(),
                        },
                        finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".to_owned()),
                        usage: TokenUsage {
                            input_tokens,
                            output_tokens,
                            total_tokens,
                        },
                    },
                })
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
        "stream": false
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

fn to_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use gunmetal_core::{
        ChatRole, ProviderAuthState, ProviderKind, ProviderProfile, RequestOptions,
    };
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, header, method, path},
    };

    use super::{ZenClient, ZenClientOptions};

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
}
