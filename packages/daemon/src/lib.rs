use std::{convert::Infallible, net::SocketAddr, time::Instant};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ChatMessage, ChatRole, GunmetalKey, KeyScope,
    NewRequestLogEntry, ProviderProfile, TokenUsage,
};
use gunmetal_providers::ProviderHub;
use gunmetal_storage::{AppPaths, StorageHandle};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_stream::iter;
use uuid::Uuid;

#[derive(Clone)]
pub struct DaemonState {
    pub paths: AppPaths,
    pub storage: StorageHandle,
    pub providers: ProviderHub,
    pub version: String,
}

impl DaemonState {
    pub fn new(paths: AppPaths) -> Result<Self> {
        let storage = paths.storage_handle()?;
        let providers = ProviderHub::new(paths.clone());
        Ok(Self {
            paths,
            storage,
            providers,
            version: env!("CARGO_PKG_VERSION").to_owned(),
        })
    }

    pub fn with_provider_hub(paths: AppPaths, providers: ProviderHub) -> Result<Self> {
        let storage = paths.storage_handle()?;
        Ok(Self {
            paths,
            storage,
            providers,
            version: env!("CARGO_PKG_VERSION").to_owned(),
        })
    }
}

pub fn app(state: DaemonState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/responses", post(responses))
        .with_state(state)
}

pub async fn serve(addr: SocketAddr, state: DaemonState) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app(state)).await?;
    Ok(())
}

async fn health(State(state): State<DaemonState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "gunmetal",
        version: state.version,
    })
}

async fn list_models(State(state): State<DaemonState>, headers: HeaderMap) -> Response {
    let key = match authorize(&state, &headers, KeyScope::ModelsRead) {
        Ok(key) => key,
        Err(error) => return error.into_response(),
    };

    let models = match state.storage.list_models() {
        Ok(models) => models,
        Err(error) => return internal_error(error),
    };

    let visible_models = models
        .into_iter()
        .filter(|model| key.can_access_provider(&model.provider))
        .map(ModelResponse::from)
        .collect::<Vec<_>>();

    Json(ModelListResponse {
        object: "list",
        data: visible_models,
    })
    .into_response()
}

async fn chat_completions(
    State(state): State<DaemonState>,
    headers: HeaderMap,
    Json(payload): Json<IncomingChatCompletionsRequest>,
) -> Response {
    let payload = match payload.validate() {
        Ok(payload) => payload,
        Err(error) => return error.into_response(),
    };

    let (key, profile, request) = match prepare_request(
        &state,
        &headers,
        payload.model,
        payload.messages,
        payload.stream,
    ) {
        Ok(context) => context,
        Err(error) => return error.into_response(),
    };

    match invoke_provider(&state, key, profile, request, "/v1/chat/completions").await {
        Ok(result) if payload.stream => streaming_chat_completion(result),
        Ok(result) => Json(outgoing_chat_completion(result)).into_response(),
        Err(response) => response,
    }
}

async fn responses(
    State(state): State<DaemonState>,
    headers: HeaderMap,
    Json(payload): Json<IncomingResponsesRequest>,
) -> Response {
    let payload = match payload.validate() {
        Ok(payload) => payload,
        Err(error) => return error.into_response(),
    };

    let (key, profile, request) = match prepare_request(
        &state,
        &headers,
        payload.model,
        payload.messages,
        payload.stream,
    ) {
        Ok(context) => context,
        Err(error) => return error.into_response(),
    };

    match invoke_provider(&state, key, profile, request, "/v1/responses").await {
        Ok(result) if payload.stream => streaming_response(result),
        Ok(result) => Json(outgoing_response(result)).into_response(),
        Err(response) => response,
    }
}

fn authorize(
    state: &DaemonState,
    headers: &HeaderMap,
    required_scope: KeyScope,
) -> Result<gunmetal_core::GunmetalKey, ApiError> {
    let secret = bearer_token(headers)?;
    let key = state
        .storage
        .authenticate_key(&secret)
        .map_err(internal_api_error)?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "invalid_api_key",
                "invalid or expired gunmetal key".to_owned(),
            )
        })?;

    if !key.scopes.contains(&required_scope) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "insufficient_scope",
            format!(
                "key '{}' does not include scope '{}'",
                key.name, required_scope
            ),
        ));
    }

    Ok(key)
}

fn bearer_token(headers: &HeaderMap) -> Result<String, ApiError> {
    let Some(value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "missing_api_key",
            "missing Authorization header".to_owned(),
        ));
    };

    let header = value.to_str().map_err(|_| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "authorization header must be valid utf-8".to_owned(),
        )
    })?;

    let Some(token) = header.strip_prefix("Bearer ") else {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "authorization header must use Bearer auth".to_owned(),
        ));
    };

    Ok(token.to_owned())
}

fn internal_error(error: impl std::fmt::Display) -> Response {
    internal_api_error(error).into_response()
}

fn internal_api_error(error: impl std::fmt::Display) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        format!("internal error: {error}"),
    )
}

fn api_error(status: StatusCode, code: &'static str, message: String) -> Response {
    ApiError::new(status, code, message).into_response()
}

fn outgoing_chat_completion(result: ChatCompletionResult) -> OutgoingChatCompletionResponse {
    OutgoingChatCompletionResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4().simple()),
        object: "chat.completion",
        created: chrono::Utc::now().timestamp(),
        model: result.model,
        choices: vec![OutgoingChoice {
            index: 0,
            message: result.message,
            finish_reason: result.finish_reason,
        }],
        usage: result.usage,
    }
}

fn outgoing_response(result: ChatCompletionResult) -> OutgoingResponsesResponse {
    let response_id = format!("resp_{}", Uuid::new_v4().simple());
    let message_id = format!("msg_{}", Uuid::new_v4().simple());
    let output_text = result.message.content.clone();
    OutgoingResponsesResponse {
        id: response_id,
        object: "response",
        created_at: chrono::Utc::now().timestamp(),
        status: "completed",
        model: result.model,
        output: vec![OutgoingResponseItem {
            id: message_id,
            item_type: "message",
            status: "completed",
            role: "assistant",
            content: vec![OutgoingResponseContent {
                content_type: "output_text",
                text: output_text,
                annotations: Vec::new(),
            }],
        }],
        usage: OutgoingResponseUsage::from(result.usage),
    }
}

fn streaming_chat_completion(result: ChatCompletionResult) -> Response {
    let id = format!("chatcmpl-{}", Uuid::new_v4().simple());
    let created = chrono::Utc::now().timestamp();
    let mut events = Vec::new();

    events.push(Ok::<Event, Infallible>(
        Event::default().data(
            json!({
                "id": id,
                "object": "chat.completion.chunk",
                "created": created,
                "model": result.model,
                "choices": [{
                    "index": 0,
                    "delta": { "role": "assistant" },
                    "finish_reason": Value::Null
                }]
            })
            .to_string(),
        ),
    ));

    for chunk in text_chunks(&result.message.content) {
        events.push(Ok::<Event, Infallible>(
            Event::default().data(
                json!({
                    "id": id,
                    "object": "chat.completion.chunk",
                    "created": created,
                    "model": result.model,
                    "choices": [{
                        "index": 0,
                        "delta": { "content": chunk },
                        "finish_reason": Value::Null
                    }]
                })
                .to_string(),
            ),
        ));
    }

    events.push(Ok::<Event, Infallible>(
        Event::default().data(
            json!({
                "id": id,
                "object": "chat.completion.chunk",
                "created": created,
                "model": result.model,
                "choices": [{
                    "index": 0,
                    "delta": {},
                    "finish_reason": result.finish_reason
                }],
                "usage": result.usage
            })
            .to_string(),
        ),
    ));
    events.push(Ok::<Event, Infallible>(Event::default().data("[DONE]")));

    Sse::new(iter(events))
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn streaming_response(result: ChatCompletionResult) -> Response {
    let response_id = format!("resp_{}", Uuid::new_v4().simple());
    let message_id = format!("msg_{}", Uuid::new_v4().simple());
    let created_at = chrono::Utc::now().timestamp();
    let mut events = Vec::new();

    events.push(Ok::<Event, Infallible>(
        Event::default().event("response.created").data(
            json!({
                "type": "response.created",
                "response": {
                    "id": response_id,
                    "object": "response",
                    "created_at": created_at,
                    "status": "in_progress",
                    "model": result.model
                }
            })
            .to_string(),
        ),
    ));

    for chunk in text_chunks(&result.message.content) {
        events.push(Ok::<Event, Infallible>(
            Event::default().event("response.output_text.delta").data(
                json!({
                    "type": "response.output_text.delta",
                    "response_id": response_id,
                    "item_id": message_id,
                    "delta": chunk
                })
                .to_string(),
            ),
        ));
    }

    events.push(Ok::<Event, Infallible>(
        Event::default().event("response.completed").data(
            json!({
                "type": "response.completed",
                "response": outgoing_response(result)
            })
            .to_string(),
        ),
    ));
    events.push(Ok::<Event, Infallible>(Event::default().data("[DONE]")));

    Sse::new(iter(events))
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn text_chunks(value: &str) -> Vec<String> {
    if value.is_empty() {
        return vec![String::new()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut count = 0usize;
    for ch in value.chars() {
        current.push(ch);
        count += 1;
        if count >= 24 {
            chunks.push(std::mem::take(&mut current));
            count = 0;
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: String,
}

#[derive(Debug, Serialize)]
struct ModelListResponse {
    object: &'static str,
    data: Vec<ModelResponse>,
}

#[derive(Debug, Serialize)]
struct ModelResponse {
    id: String,
    object: &'static str,
    owned_by: String,
    provider: String,
}

impl From<gunmetal_core::ModelDescriptor> for ModelResponse {
    fn from(value: gunmetal_core::ModelDescriptor) -> Self {
        let owned_by = value.provider.to_string();
        Self {
            id: value.id,
            object: "model",
            owned_by: owned_by.clone(),
            provider: owned_by,
        }
    }
}

#[derive(Debug, Deserialize)]
struct IncomingChatCompletionsRequest {
    model: String,
    messages: Vec<IncomingChatMessage>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct IncomingResponsesRequest {
    model: String,
    instructions: Option<String>,
    input: Option<IncomingResponsesInput>,
    stream: Option<bool>,
}

impl IncomingResponsesRequest {
    fn validate(self) -> Result<ValidatedChatRequest, ApiError> {
        if self.model.trim().is_empty() {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "model is required".to_owned(),
            ));
        }

        let mut messages = Vec::new();
        if let Some(instructions) = self.instructions
            && !instructions.trim().is_empty()
        {
            messages.push(ChatMessage {
                role: ChatRole::System,
                content: instructions,
            });
        }

        let input = self.input.ok_or_else(|| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "input is required".to_owned(),
            )
        })?;

        match input {
            IncomingResponsesInput::Text(text) => {
                if text.trim().is_empty() {
                    return Err(ApiError::new(
                        StatusCode::BAD_REQUEST,
                        "invalid_request",
                        "input text cannot be empty".to_owned(),
                    ));
                }
                messages.push(ChatMessage {
                    role: ChatRole::User,
                    content: text,
                });
            }
            IncomingResponsesInput::Items(items) => {
                for item in items {
                    let message = item.into_message()?;
                    if message.content.trim().is_empty() {
                        return Err(ApiError::new(
                            StatusCode::BAD_REQUEST,
                            "invalid_request",
                            "input content cannot be empty".to_owned(),
                        ));
                    }
                    messages.push(message);
                }
            }
        }

        if messages.is_empty() {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "input is required".to_owned(),
            ));
        }

        Ok(ValidatedChatRequest {
            model: self.model,
            messages,
            stream: self.stream.unwrap_or(false),
        })
    }
}

impl IncomingChatCompletionsRequest {
    fn validate(self) -> Result<ValidatedChatRequest, ApiError> {
        if self.model.trim().is_empty() {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "model is required".to_owned(),
            ));
        }
        if self.messages.is_empty() {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "at least one message is required".to_owned(),
            ));
        }

        let mut messages = Vec::with_capacity(self.messages.len());
        for message in self.messages {
            if message.content.trim().is_empty() {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "message content cannot be empty".to_owned(),
                ));
            }
            let role = message.role.parse::<ChatRole>().map_err(|error| {
                ApiError::new(StatusCode::BAD_REQUEST, "invalid_request", error)
            })?;
            messages.push(ChatMessage {
                role,
                content: message.content,
            });
        }

        Ok(ValidatedChatRequest {
            model: self.model,
            messages,
            stream: self.stream.unwrap_or(false),
        })
    }
}

#[derive(Debug, Deserialize)]
struct IncomingChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncomingResponsesInput {
    Text(String),
    Items(Vec<IncomingResponsesItem>),
}

#[derive(Debug, Deserialize)]
struct IncomingResponsesItem {
    role: Option<String>,
    content: IncomingResponsesContent,
}

impl IncomingResponsesItem {
    fn into_message(self) -> Result<ChatMessage, ApiError> {
        let role = match self.role.as_deref().unwrap_or("user") {
            "developer" | "system" => ChatRole::System,
            "user" => ChatRole::User,
            "assistant" => ChatRole::Assistant,
            value => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    format!("unknown chat role: {value}"),
                ));
            }
        };

        Ok(ChatMessage {
            role,
            content: self.content.into_text()?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncomingResponsesContent {
    Text(String),
    Parts(Vec<IncomingResponsesContentPart>),
}

impl IncomingResponsesContent {
    fn into_text(self) -> Result<String, ApiError> {
        match self {
            Self::Text(text) => Ok(text),
            Self::Parts(parts) => {
                let mut text_parts = Vec::new();
                for part in parts {
                    match part.kind.as_str() {
                        "input_text" | "text" | "output_text" => {
                            let Some(text) = part.text else {
                                return Err(ApiError::new(
                                    StatusCode::BAD_REQUEST,
                                    "invalid_request",
                                    "text content part is missing text".to_owned(),
                                ));
                            };
                            if !text.trim().is_empty() {
                                text_parts.push(text);
                            }
                        }
                        value => {
                            return Err(ApiError::new(
                                StatusCode::BAD_REQUEST,
                                "invalid_request",
                                format!("unsupported response content part: {value}"),
                            ));
                        }
                    }
                }
                Ok(text_parts.join("\n"))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct IncomingResponsesContentPart {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Debug, Clone)]
struct ValidatedChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OutgoingChatCompletionResponse {
    id: String,
    object: &'static str,
    created: i64,
    model: String,
    choices: Vec<OutgoingChoice>,
    usage: TokenUsage,
}

#[derive(Debug, Serialize)]
struct OutgoingChoice {
    index: usize,
    message: ChatMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct OutgoingResponsesResponse {
    id: String,
    object: &'static str,
    created_at: i64,
    status: &'static str,
    model: String,
    output: Vec<OutgoingResponseItem>,
    usage: OutgoingResponseUsage,
}

#[derive(Debug, Serialize)]
struct OutgoingResponseItem {
    id: String,
    #[serde(rename = "type")]
    item_type: &'static str,
    status: &'static str,
    role: &'static str,
    content: Vec<OutgoingResponseContent>,
}

#[derive(Debug, Serialize)]
struct OutgoingResponseContent {
    #[serde(rename = "type")]
    content_type: &'static str,
    text: String,
    annotations: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct OutgoingResponseUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

impl From<TokenUsage> for OutgoingResponseUsage {
    fn from(value: TokenUsage) -> Self {
        Self {
            input_tokens: value.input_tokens,
            output_tokens: value.output_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

#[derive(Debug, Clone)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: String) -> Self {
        Self {
            status,
            code,
            message,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "error": {
                    "message": self.message,
                    "type": self.code,
                    "code": self.code,
                }
            })),
        )
            .into_response()
    }
}

fn prepare_request(
    state: &DaemonState,
    headers: &HeaderMap,
    model_id: String,
    messages: Vec<ChatMessage>,
    stream: bool,
) -> Result<(GunmetalKey, ProviderProfile, ChatCompletionRequest), ApiError> {
    let key = authorize(state, headers, KeyScope::Inference)?;

    let models = state.storage.list_models().map_err(internal_api_error)?;
    let Some(model) = models.into_iter().find(|item| item.id == model_id) else {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "model_not_found",
            format!("model '{}' is not registered in gunmetal", model_id),
        ));
    };

    if !key.can_access_provider(&model.provider) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "provider_forbidden",
            format!(
                "key '{}' cannot access provider '{}'",
                key.name, model.provider
            ),
        ));
    }

    let Some(profile_id) = model.profile_id else {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "profile_missing",
            format!("model '{}' is not attached to a provider profile", model.id),
        ));
    };

    let Some(profile) = state
        .storage
        .get_profile(profile_id)
        .map_err(internal_api_error)?
    else {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "profile_missing",
            format!("profile '{}' does not exist", profile_id),
        ));
    };

    Ok((
        key,
        profile,
        ChatCompletionRequest {
            model: model.id,
            messages,
            stream,
        },
    ))
}

async fn invoke_provider(
    state: &DaemonState,
    key: GunmetalKey,
    profile: ProviderProfile,
    request: ChatCompletionRequest,
    endpoint: &'static str,
) -> Result<ChatCompletionResult, Response> {
    let started_at = Instant::now();
    match state.providers.chat_completion(&profile, &request).await {
        Ok(result) => {
            let duration_ms = started_at.elapsed().as_millis() as u64;
            let _ = state.storage.log_request(NewRequestLogEntry {
                key_id: Some(key.id),
                profile_id: Some(profile.id),
                provider: profile.provider,
                model: request.model,
                endpoint: endpoint.to_owned(),
                status_code: Some(StatusCode::OK.as_u16()),
                duration_ms,
                usage: result.usage.clone(),
                error_message: None,
            });
            Ok(result)
        }
        Err(error) => {
            let duration_ms = started_at.elapsed().as_millis() as u64;
            let _ = state.storage.log_request(NewRequestLogEntry {
                key_id: Some(key.id),
                profile_id: Some(profile.id),
                provider: profile.provider,
                model: request.model,
                endpoint: endpoint.to_owned(),
                status_code: Some(StatusCode::BAD_GATEWAY.as_u16()),
                duration_ms,
                usage: TokenUsage {
                    input_tokens: None,
                    output_tokens: None,
                    total_tokens: None,
                },
                error_message: Some(error.to_string()),
            });
            Err(api_error(
                StatusCode::BAD_GATEWAY,
                "provider_request_failed",
                format!("provider request failed: {error}"),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{future::Future, pin::Pin};

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        response::Response,
    };
    use gunmetal_core::{KeyScope, NewGunmetalKey, NewProviderProfile, ProviderKind};
    use gunmetal_providers::{CodexClient, ProviderHub};
    use gunmetal_storage::{AppPaths, StorageHandle};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tower::util::ServiceExt;

    use super::{DaemonState, app};

    #[tokio::test]
    async fn health_endpoint_is_live() {
        let fixture = Fixture::new();
        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "gunmetal");
    }

    #[tokio::test]
    async fn models_endpoint_requires_valid_key_and_scope() {
        let fixture = Fixture::new();
        fixture.seed_models();

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let limited = fixture.create_key(vec![KeyScope::Inference], vec![]);
        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .header(header::AUTHORIZATION, format!("Bearer {limited}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn models_endpoint_filters_by_provider_scope() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::ModelsRead], vec![ProviderKind::Codex]);

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["data"].as_array().unwrap().len(), 1);
        assert_eq!(body["data"][0]["id"], "codex/gpt-5.4");
    }

    #[tokio::test]
    async fn chat_completion_rejects_unknown_model() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![]);

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "model": "codex/does-not-exist",
                            "messages": [{ "role": "user", "content": "ping" }]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_json(response).await;
        assert_eq!(body["error"]["code"], "model_not_found");
    }

    #[tokio::test]
    async fn chat_completion_calls_provider_and_logs_success() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![ProviderKind::Codex]);

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "model": "codex/gpt-5.4",
                            "messages": [{ "role": "user", "content": "ping" }],
                            "stream": false
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["choices"][0]["message"]["content"], "hello from codex");

        let logs = fixture.storage.list_request_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status_code, Some(200));
    }

    #[tokio::test]
    async fn chat_completion_streams_sse_chunks() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![ProviderKind::Codex]);

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "model": "codex/gpt-5.4",
                            "messages": [{ "role": "user", "content": "ping" }],
                            "stream": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_text(response).await;
        assert!(body.contains("chat.completion.chunk"));
        assert!(body.contains("\"role\":\"assistant\""));
        assert!(body.contains("hello from codex"));
        assert!(body.contains("[DONE]"));
    }

    #[tokio::test]
    async fn responses_rejects_missing_input() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![]);

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/responses")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "model": "codex/gpt-5.4"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_json(response).await;
        assert_eq!(body["error"]["code"], "invalid_request");
    }

    #[tokio::test]
    async fn responses_calls_provider_and_logs_success() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![ProviderKind::Codex]);

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/responses")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "model": "codex/gpt-5.4",
                            "instructions": "be terse",
                            "input": "ping"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["object"], "response");
        assert_eq!(body["status"], "completed");
        assert_eq!(body["output"][0]["type"], "message");
        assert_eq!(body["output"][0]["role"], "assistant");
        assert_eq!(body["output"][0]["content"][0]["type"], "output_text");
        assert_eq!(body["output"][0]["content"][0]["text"], "hello from codex");

        let logs = fixture.storage.list_request_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status_code, Some(200));
        assert_eq!(logs[0].endpoint, "/v1/responses");
    }

    #[tokio::test]
    async fn responses_stream_sse_events() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![ProviderKind::Codex]);

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/responses")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "model": "codex/gpt-5.4",
                            "input": "ping",
                            "stream": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_text(response).await;
        assert!(body.contains("event: response.created"));
        assert!(body.contains("event: response.output_text.delta"));
        assert!(body.contains("event: response.completed"));
        assert!(body.contains("hello from codex"));
        assert!(body.contains("[DONE]"));
    }

    struct Fixture {
        _temp: TempDir,
        paths: AppPaths,
        storage: StorageHandle,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TempDir::new().unwrap();
            let paths = AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
            let storage = paths.storage_handle().unwrap();
            Self {
                _temp: temp,
                paths,
                storage,
            }
        }

        fn state(&self) -> DaemonState {
            let connector = Arc::new(
                move |_profile: gunmetal_core::ProviderProfile,
                      _paths: AppPaths|
                      -> Pin<
                    Box<dyn Future<Output = anyhow::Result<CodexClient>> + Send>,
                > { Box::pin(async move { Ok(CodexClient::mock("hello from codex")) }) },
            );
            let providers = ProviderHub::with_codex_connector(self.paths.clone(), connector);
            DaemonState::with_provider_hub(self.paths.clone(), providers).unwrap()
        }

        fn create_key(
            &self,
            scopes: Vec<KeyScope>,
            allowed_providers: Vec<ProviderKind>,
        ) -> String {
            self.storage
                .create_key(NewGunmetalKey {
                    name: "test".to_owned(),
                    scopes,
                    allowed_providers,
                    expires_at: None,
                })
                .unwrap()
                .secret
        }

        fn seed_models(&self) {
            let profile = self
                .storage
                .create_profile(NewProviderProfile {
                    provider: ProviderKind::Codex,
                    name: "default".to_owned(),
                    base_url: None,
                    enabled: true,
                    credentials: None,
                })
                .unwrap();

            self.storage
                .replace_models_for_profile(
                    &ProviderKind::Codex,
                    Some(profile.id),
                    &[gunmetal_core::ModelDescriptor {
                        id: "codex/gpt-5.4".to_owned(),
                        provider: ProviderKind::Codex,
                        profile_id: Some(profile.id),
                        upstream_name: "gpt-5.4".to_owned(),
                        display_name: "GPT-5.4".to_owned(),
                    }],
                )
                .unwrap();
        }
    }

    async fn to_json(response: Response) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn to_text(response: Response) -> String {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }
}
