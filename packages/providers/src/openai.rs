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

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiClientOptions {
    api_key: Option<String>,
    base_url: String,
}

impl OpenAiClientOptions {
    pub fn from_profile(profile: &ProviderProfile) -> Self {
        let settings = profile
            .credentials
            .clone()
            .and_then(|value| serde_json::from_value::<OpenAiProfileSettings>(value).ok())
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
        OpenAiProfileSettings {
            api_key,
            base_url: (self.base_url != DEFAULT_BASE_URL).then(|| self.base_url.clone()),
        }
        .into_value()
    }
}

#[derive(Clone)]
pub struct OpenAiClient {
    http: Client,
    mode: OpenAiMode,
}

#[derive(Clone)]
enum OpenAiMode {
    Live(OpenAiClientOptions),
    Mock(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OpenAiProfileSettings {
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
}

impl OpenAiProfileSettings {
    fn into_value(self) -> Option<Value> {
        let is_empty = self.api_key.is_none() && self.base_url.is_none();
        if is_empty {
            None
        } else {
            Some(serde_json::to_value(self).expect("serialize openai credentials"))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ModelListResponse {
    data: Vec<OpenAiModelRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiModelRecord {
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
    status: u16,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ApiError {}

impl OpenAiClient {
    pub fn with_options(options: OpenAiClientOptions) -> Self {
        Self {
            http: Client::builder().build().expect("reqwest client"),
            mode: OpenAiMode::Live(options),
        }
    }

    pub fn mock(response: impl Into<String>) -> Self {
        Self {
            http: Client::builder().build().expect("reqwest client"),
            mode: OpenAiMode::Mock(response.into()),
        }
    }

    pub async fn auth_status(&self, _profile: &ProviderProfile) -> Result<ProviderAuthStatus> {
        match &self.mode {
            OpenAiMode::Mock(_) => Ok(ProviderAuthStatus {
                state: ProviderAuthState::Connected,
                label: "mock@gunmetal (openai)".to_owned(),
            }),
            OpenAiMode::Live(options) => {
                let Some(api_key) = options.api_key.as_deref() else {
                    return Ok(ProviderAuthStatus {
                        state: ProviderAuthState::SignedOut,
                        label: "Missing OpenAI API key".to_owned(),
                    });
                };

                let response = self
                    .http
                    .get(format!("{}/models", options.base_url))
                    .headers(self.headers(api_key)?)
                    .send()
                    .await?;

                if !response.status().is_success() {
                    let error = read_error(response).await;
                    let state = if error.status == 401 {
                        ProviderAuthState::Error
                    } else {
                        ProviderAuthState::Expired
                    };
                    return Ok(ProviderAuthStatus {
                        state,
                        label: error.message,
                    });
                }

                let _payload: ModelListResponse = response.json().await?;
                Ok(ProviderAuthStatus {
                    state: ProviderAuthState::Connected,
                    label: "OpenAI API key".to_owned(),
                })
            }
        }
    }

    pub fn clear_credentials(&self) -> Option<Value> {
        match &self.mode {
            OpenAiMode::Mock(_) => None,
            OpenAiMode::Live(options) => options.persisted_credentials_with_api_key(None),
        }
    }

    pub async fn list_models(&self, profile: &ProviderProfile) -> Result<Vec<ModelDescriptor>> {
        match &self.mode {
            OpenAiMode::Mock(_) => Ok(vec![ModelDescriptor {
                id: "openai/gpt-5.1".to_owned(),
                provider: ProviderKind::OpenAi,
                profile_id: Some(profile.id),
                upstream_name: "gpt-5.1".to_owned(),
                display_name: "gpt-5.1".to_owned(),
                metadata: None,
            }]),
            OpenAiMode::Live(options) => {
                let api_key = Self::api_key(options)?;
                let response = self
                    .http
                    .get(format!("{}/models", options.base_url))
                    .headers(self.headers(api_key)?)
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
                            id: format!("openai/{upstream_name}"),
                            provider: ProviderKind::OpenAi,
                            profile_id: Some(profile.id),
                            display_name: upstream_name.clone(),
                            upstream_name,
                            metadata: None,
                        }
                    })
                    .collect::<Vec<_>>();
                models.sort_by(|left, right| left.id.cmp(&right.id));
                Ok(models)
            }
        }
    }

    pub async fn chat_completion(
        &self,
        _profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResult> {
        match &self.mode {
            OpenAiMode::Mock(response) => Ok(ChatCompletionResult {
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
            }),
            OpenAiMode::Live(options) => {
                let api_key = Self::api_key(options)?;
                let model = request
                    .model
                    .strip_prefix("openai/")
                    .unwrap_or(&request.model)
                    .to_owned();

                let response = self
                    .http
                    .post(format!("{}/chat/completions", options.base_url))
                    .headers(self.headers(api_key)?)
                    .json(&build_openai_request_body(&model, request))
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
                    .ok_or_else(|| anyhow!("openai returned no choices"))?;
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

                Ok(ChatCompletionResult {
                    model: format!(
                        "openai/{}",
                        payload.model.unwrap_or_else(|| model.to_owned())
                    ),
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
                })
            }
        }
    }

    fn api_key(options: &OpenAiClientOptions) -> Result<&str> {
        options
            .api_key
            .as_deref()
            .ok_or_else(|| anyhow!("openai api key missing"))
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
    let status = response.status().as_u16();
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
        .unwrap_or("openai request failed");

    ApiError {
        message: message.to_owned(),
        status,
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

fn build_openai_request_body(model: &str, request: &ChatCompletionRequest) -> Value {
    let mut body = json!({
        "model": model,
        "messages": request.messages.iter().map(to_upstream_message).collect::<Vec<_>>(),
        "stream": false
    });
    let object = body.as_object_mut().expect("openai request object");

    if let Some(value) = request.options.temperature {
        object.insert("temperature".to_owned(), json!(value));
    }
    if let Some(value) = request.options.top_p {
        object.insert("top_p".to_owned(), json!(value));
    }
    if let Some(value) = request.options.max_output_tokens {
        object.insert("max_completion_tokens".to_owned(), json!(value));
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
        ChatRole, ProviderAuthState, ProviderKind, ProviderProfile, RequestMode, RequestOptions,
    };
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, header, method, path},
    };

    use super::{OpenAiClient, OpenAiClientOptions};

    #[tokio::test]
    async fn missing_key_is_signed_out() {
        let profile = ProviderProfile {
            id: uuid::Uuid::new_v4(),
            provider: ProviderKind::OpenAi,
            name: "openai".to_owned(),
            base_url: None,
            enabled: true,
            credentials: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let client = OpenAiClient::with_options(OpenAiClientOptions::from_profile(&profile));

        let status = client.auth_status(&profile).await.unwrap();
        assert_eq!(status.state, ProviderAuthState::SignedOut);
        assert_eq!(status.label, "Missing OpenAI API key");
    }

    #[tokio::test]
    async fn validates_key_lists_models_and_completes_chat() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .and(header("authorization", "Bearer sk-openai-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "object": "list",
                "data": [
                    { "id": "gpt-5.1" },
                    { "id": "gpt-4.1-mini" }
                ]
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer sk-openai-test"))
            .and(body_string_contains("\"model\":\"gpt-5.1\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "model": "gpt-5.1",
                "choices": [{
                    "finish_reason": "stop",
                    "message": { "content": "GUNMETAL_OPENAI_OK" }
                }],
                "usage": {
                    "prompt_tokens": 6,
                    "completion_tokens": 2,
                    "total_tokens": 8
                }
            })))
            .mount(&server)
            .await;

        let profile = ProviderProfile {
            id: uuid::Uuid::new_v4(),
            provider: ProviderKind::OpenAi,
            name: "openai".to_owned(),
            base_url: Some(server.uri()),
            enabled: true,
            credentials: Some(json!({
                "api_key": "sk-openai-test"
            })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let client = OpenAiClient::with_options(OpenAiClientOptions::from_profile(&profile));

        let status = client.auth_status(&profile).await.unwrap();
        assert_eq!(status.state, ProviderAuthState::Connected);
        assert_eq!(status.label, "OpenAI API key");

        let models = client.list_models(&profile).await.unwrap();
        assert_eq!(models[0].id, "openai/gpt-4.1-mini");
        assert_eq!(models[1].id, "openai/gpt-5.1");

        let completion = client
            .chat_completion(
                &profile,
                &gunmetal_core::ChatCompletionRequest {
                    model: "openai/gpt-5.1".to_owned(),
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
        assert_eq!(completion.message.content, "GUNMETAL_OPENAI_OK");
        assert_eq!(completion.usage.total_tokens, Some(8));
    }

    #[tokio::test]
    async fn passthrough_mode_merges_provider_options() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "model": "gpt-5.1",
                "choices": [{
                    "finish_reason": "stop",
                    "message": { "content": "GUNMETAL_OPENAI_OK" }
                }],
                "usage": {
                    "prompt_tokens": 6,
                    "completion_tokens": 2,
                    "total_tokens": 8
                }
            })))
            .mount(&server)
            .await;

        let profile = ProviderProfile {
            id: uuid::Uuid::new_v4(),
            provider: ProviderKind::OpenAi,
            name: "openai".to_owned(),
            base_url: Some(server.uri()),
            enabled: true,
            credentials: Some(json!({
                "api_key": "sk-openai-test"
            })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let client = OpenAiClient::with_options(OpenAiClientOptions::from_profile(&profile));

        let completion = client
            .chat_completion(
                &profile,
                &gunmetal_core::ChatCompletionRequest {
                    model: "openai/gpt-5.1".to_owned(),
                    messages: vec![gunmetal_core::ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: false,
                    options: RequestOptions {
                        temperature: Some(0.2),
                        top_p: Some(0.9),
                        max_output_tokens: Some(128),
                        stop: vec!["DONE".to_owned()],
                        metadata: serde_json::Map::from_iter([(
                            "trace_id".to_owned(),
                            json!("abc"),
                        )]),
                        provider_options: serde_json::Map::from_iter([(
                            "reasoning".to_owned(),
                            json!({ "effort": "high" }),
                        )]),
                        mode: RequestMode::Passthrough,
                    },
                },
            )
            .await
            .unwrap();

        assert_eq!(completion.message.content, "GUNMETAL_OPENAI_OK");
        let requests = server.received_requests().await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert!(
            body["temperature"]
                .as_f64()
                .is_some_and(|value| (value - 0.2).abs() < 0.0001)
        );
        assert!(
            body["top_p"]
                .as_f64()
                .is_some_and(|value| (value - 0.9).abs() < 0.0001)
        );
        assert_eq!(body["max_completion_tokens"], 128);
        assert_eq!(body["stop"], json!(["DONE"]));
        assert_eq!(body["metadata"]["trace_id"], "abc");
        assert_eq!(body["reasoning"]["effort"], "high");
    }
}
