use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::mpsc::{self, Sender},
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use anyhow::Result;
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{
        Html, IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{delete, get, post},
};
use futures_util::{StreamExt, stream};
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ChatMessage, ChatRole, GunmetalKey, KeyScope,
    KeyState, ModelDescriptor, NewGunmetalKey, NewProviderProfile, NewRequestLogEntry,
    ProviderKind, ProviderProfile, RequestMode, RequestOptions, TokenUsage,
};
use gunmetal_providers::{builtin_provider_hub, builtin_providers};
use gunmetal_sdk::{
    ProviderByteStream, ProviderClass, ProviderEventStream, ProviderHub, ProviderStreamEvent,
};
use gunmetal_storage::{AppPaths, StorageHandle};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

const BROWSER_APP_HTML: &str = include_str!("browser_app.html");

#[derive(Clone)]
pub struct DaemonState {
    pub paths: AppPaths,
    pub storage: StorageHandle,
    pub providers: ProviderHub,
    pub version: String,
    request_logger: RequestLogger,
    request_cache: RequestCache,
}

impl DaemonState {
    pub fn new(paths: AppPaths) -> Result<Self> {
        let storage = paths.storage_handle()?;
        let providers = builtin_provider_hub(paths.clone());
        let request_logger = RequestLogger::new(storage.clone());
        Ok(Self {
            paths,
            storage,
            providers,
            version: env!("CARGO_PKG_VERSION").to_owned(),
            request_logger,
            request_cache: RequestCache::default(),
        })
    }

    pub fn with_provider_hub(paths: AppPaths, providers: ProviderHub) -> Result<Self> {
        let storage = paths.storage_handle()?;
        let request_logger = RequestLogger::new(storage.clone());
        Ok(Self {
            paths,
            storage,
            providers,
            version: env!("CARGO_PKG_VERSION").to_owned(),
            request_logger,
            request_cache: RequestCache::default(),
        })
    }
}

#[derive(Clone)]
struct RequestLogger {
    sender: Sender<NewRequestLogEntry>,
}

#[derive(Clone, Default)]
struct RequestCache {
    models: Arc<Mutex<HashMap<String, ModelDescriptor>>>,
    profiles: Arc<Mutex<HashMap<Uuid, ProviderProfile>>>,
}

impl RequestCache {
    fn model(&self, id: &str) -> Option<ModelDescriptor> {
        self.models.lock().unwrap().get(id).cloned()
    }

    fn insert_model(&self, model: ModelDescriptor) {
        self.models.lock().unwrap().insert(model.id.clone(), model);
    }

    fn profile(&self, id: Uuid) -> Option<ProviderProfile> {
        self.profiles.lock().unwrap().get(&id).cloned()
    }

    fn insert_profile(&self, profile: ProviderProfile) {
        self.profiles.lock().unwrap().insert(profile.id, profile);
    }

    fn clear(&self) {
        self.models.lock().unwrap().clear();
        self.profiles.lock().unwrap().clear();
    }
}

impl RequestLogger {
    fn new(storage: StorageHandle) -> Self {
        let (sender, receiver) = mpsc::channel::<NewRequestLogEntry>();
        thread::Builder::new()
            .name("gunmetal-request-logger".to_owned())
            .spawn(move || {
                while let Ok(entry) = receiver.recv() {
                    let _ = storage.log_request(entry);
                }
            })
            .expect("spawn request logger");
        Self { sender }
    }

    fn log(&self, entry: NewRequestLogEntry) {
        let _ = self.sender.send(entry);
    }
}

pub fn app(state: DaemonState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/app", get(browser_app))
        .route("/app/api/state", get(operator_state))
        .route("/app/api/profiles", post(create_profile))
        .route("/app/api/profiles/{id}/auth", post(auth_profile))
        .route("/app/api/profiles/{id}/sync", post(sync_profile))
        .route("/app/api/profiles/{id}/logout", post(logout_profile))
        .route("/app/api/profiles/{id}/keys", post(create_profile_key))
        .route("/app/api/profiles/{id}", delete(delete_profile))
        .route("/app/api/keys/{id}/state", post(set_key_state))
        .route("/app/api/keys/{id}", delete(delete_key))
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

async fn browser_app() -> Html<&'static str> {
    Html(BROWSER_APP_HTML)
}

async fn operator_state(State(state): State<DaemonState>) -> Response {
    match load_operator_state(&state) {
        Ok(payload) => Json(payload).into_response(),
        Err(error) => internal_error(error),
    }
}

async fn create_profile(
    State(state): State<DaemonState>,
    Json(payload): Json<CreateProfilePayload>,
) -> Response {
    let provider = match payload.provider.parse::<ProviderKind>() {
        Ok(provider) => provider,
        Err(error) => return api_error(StatusCode::BAD_REQUEST, "invalid_request", error),
    };

    let name = payload.name.trim();
    if name.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "provider name is required".to_owned(),
        );
    }

    let profile = match state.storage.create_profile(NewProviderProfile {
        provider: provider.clone(),
        name: name.to_owned(),
        base_url: payload.base_url.and_then(trimmed_or_none),
        enabled: true,
        credentials: operator_profile_credentials(&provider, payload.api_key),
    }) {
        Ok(profile) => profile,
        Err(error) => return internal_error(error),
    };
    state.request_cache.clear();

    Json(OperatorActionResponse::message(format!(
        "Saved provider {} ({})",
        profile.name, profile.provider
    )))
    .into_response()
}

async fn auth_profile(State(state): State<DaemonState>, Path(id): Path<Uuid>) -> Response {
    let profile = match require_profile(&state, id) {
        Ok(profile) => profile,
        Err(error) => return error.into_response(),
    };

    if supports_browser_login(&profile.provider) {
        match state.providers.login(&profile, false).await {
            Ok(session) => {
                state.request_cache.clear();
                return Json(OperatorActionResponse {
                    message: format!("Open the browser flow for {}.", profile.name),
                    auth_url: Some(session.auth_url),
                    user_code: session.user_code,
                    secret: None,
                })
                .into_response();
            }
            Err(error) => return internal_error(error),
        }
    }

    match state.providers.auth_status(&profile).await {
        Ok(status) => {
            state.request_cache.clear();
            Json(OperatorActionResponse::message(format!(
                "Auth {:?}: {}",
                status.state, status.label
            )))
            .into_response()
        }
        Err(error) => internal_error(error),
    }
}

async fn sync_profile(State(state): State<DaemonState>, Path(id): Path<Uuid>) -> Response {
    let profile = match require_profile(&state, id) {
        Ok(profile) => profile,
        Err(error) => return error.into_response(),
    };

    match state.providers.sync_models(&profile).await {
        Ok(models) => match state.storage.replace_models_for_profile(
            &profile.provider,
            Some(profile.id),
            &models,
        ) {
            Ok(()) => {
                state.request_cache.clear();
                Json(OperatorActionResponse::message(format!(
                    "Synced {} models for {}.",
                    models.len(),
                    profile.name
                )))
                .into_response()
            }
            Err(error) => internal_error(error),
        },
        Err(error) => internal_error(error),
    }
}

async fn logout_profile(State(state): State<DaemonState>, Path(id): Path<Uuid>) -> Response {
    let profile = match require_profile(&state, id) {
        Ok(profile) => profile,
        Err(error) => return error.into_response(),
    };

    match state.providers.logout(&profile).await {
        Ok(()) => {
            state.request_cache.clear();
            Json(OperatorActionResponse::message(format!(
                "Logged out {}.",
                profile.name
            )))
            .into_response()
        }
        Err(error) => internal_error(error),
    }
}

async fn create_profile_key(
    State(state): State<DaemonState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateKeyPayload>,
) -> Response {
    let profile = match require_profile(&state, id) {
        Ok(profile) => profile,
        Err(error) => return error.into_response(),
    };

    match state.storage.create_key(NewGunmetalKey {
        name: payload
            .name
            .and_then(trimmed_or_none)
            .unwrap_or_else(|| format!("{}-key", profile.name)),
        scopes: operator_default_scopes(),
        allowed_providers: Vec::new(),
        expires_at: None,
    }) {
        Ok(created) => {
            state.request_cache.clear();
            Json(OperatorActionResponse {
                message: format!("Created key {}.", created.record.name),
                auth_url: None,
                user_code: None,
                secret: Some(created.secret),
            })
            .into_response()
        }
        Err(error) => internal_error(error),
    }
}

async fn delete_profile(State(state): State<DaemonState>, Path(id): Path<Uuid>) -> Response {
    let profile = match require_profile(&state, id) {
        Ok(profile) => profile,
        Err(error) => return error.into_response(),
    };

    match state.storage.delete_profile(profile.id) {
        Ok(()) => {
            state.request_cache.clear();
            Json(OperatorActionResponse::message(format!(
                "Deleted provider {}.",
                profile.name
            )))
            .into_response()
        }
        Err(error) => internal_error(error),
    }
}

async fn set_key_state(
    State(state): State<DaemonState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SetKeyStatePayload>,
) -> Response {
    let key = match require_key(&state, id) {
        Ok(key) => key,
        Err(error) => return error.into_response(),
    };
    let next_state = match payload.state.parse::<KeyState>() {
        Ok(state) => state,
        Err(error) => return api_error(StatusCode::BAD_REQUEST, "invalid_request", error),
    };

    match state.storage.set_key_state(key.id, next_state.clone()) {
        Ok(()) => {
            state.request_cache.clear();
            Json(OperatorActionResponse::message(format!(
                "Set {} to {}.",
                key.name, next_state
            )))
            .into_response()
        }
        Err(error) => internal_error(error),
    }
}

async fn delete_key(State(state): State<DaemonState>, Path(id): Path<Uuid>) -> Response {
    let key = match require_key(&state, id) {
        Ok(key) => key,
        Err(error) => return error.into_response(),
    };

    match state.storage.delete_key(key.id) {
        Ok(()) => {
            state.request_cache.clear();
            Json(OperatorActionResponse::message(format!(
                "Deleted key {}.",
                key.name
            )))
            .into_response()
        }
        Err(error) => internal_error(error),
    }
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
        payload.options,
    ) {
        Ok(context) => context,
        Err(error) => return error.into_response(),
    };

    if payload.stream {
        return match invoke_provider_raw_stream(
            &state,
            key,
            profile,
            request.clone(),
            "/v1/chat/completions",
        )
        .await
        {
            Ok(result) => raw_streaming_chat_completion(result),
            Err(response) => response,
        };
    }

    match invoke_provider(&state, key, profile, request, "/v1/chat/completions").await {
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
        payload.options,
    ) {
        Ok(context) => context,
        Err(error) => return error.into_response(),
    };

    if payload.stream {
        return match invoke_provider_stream(&state, key, profile, request.clone(), "/v1/responses")
            .await
        {
            Ok(result) => streaming_response(request.model, result),
            Err(response) => response,
        };
    }

    match invoke_provider(&state, key, profile, request, "/v1/responses").await {
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
                "Invalid or expired Gunmetal key. Create a new key in `gunmetal setup` or `gunmetal keys create`.".to_owned(),
            )
        })?;

    if !key.scopes.contains(&required_scope) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "insufficient_scope",
            format!(
                "Key '{}' is missing scope '{}'. Create a broader key with `gunmetal keys create`.",
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
            "Missing Authorization header. Send `Authorization: Bearer gm_...`.".to_owned(),
        ));
    };

    let header = value.to_str().map_err(|_| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "Authorization header must be valid utf-8.".to_owned(),
        )
    })?;

    let Some(token) = header.strip_prefix("Bearer ") else {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_api_key",
            "Authorization header must use Bearer auth. Send `Authorization: Bearer gm_...`."
                .to_owned(),
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
        output_text: output_text.clone(),
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

fn raw_streaming_chat_completion(provider_stream: ProviderByteStream) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/event-stream")
        .header(axum::http::header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(provider_stream.map(|item| {
            match item {
                Ok(chunk) => Ok::<Vec<u8>, Infallible>(chunk),
                Err(error) => Ok::<Vec<u8>, Infallible>(
                    format!(
                        "event: error\ndata: {}\n\ndata: [DONE]\n\n",
                        json!({ "error": { "message": error.to_string() } })
                    )
                    .into_bytes(),
                ),
            }
        })))
        .expect("chat completion stream response")
}

fn streaming_response(model: String, provider_stream: ProviderEventStream) -> Response {
    let response_id = format!("resp_{}", Uuid::new_v4().simple());
    let message_id = format!("msg_{}", Uuid::new_v4().simple());
    let created_at = chrono::Utc::now().timestamp();
    let response_id_for_created = response_id.clone();
    let model_for_created = model.clone();
    let created = stream::once(async move {
        Ok::<Event, Infallible>(
            Event::default().event("response.created").data(
                json!({
                    "type": "response.created",
                    "response": {
                        "id": response_id_for_created,
                        "object": "response",
                        "created_at": created_at,
                        "status": "in_progress",
                        "model": model_for_created
                    }
                })
                .to_string(),
            ),
        )
    });
    let response_id_for_events = response_id.clone();
    let message_id_for_events = message_id.clone();
    let mut output = String::new();
    let content = provider_stream.map(move |item| match item {
        Ok(ProviderStreamEvent::TextDelta(delta)) => {
            output.push_str(&delta);
            Ok::<Event, Infallible>(
                Event::default().event("response.output_text.delta").data(
                    json!({
                        "type": "response.output_text.delta",
                        "response_id": response_id_for_events,
                        "item_id": message_id_for_events,
                        "delta": delta
                    })
                    .to_string(),
                ),
            )
        }
        Ok(ProviderStreamEvent::Complete {
            model,
            finish_reason,
            usage,
        }) => Ok::<Event, Infallible>(
            Event::default().event("response.completed").data(
                json!({
                    "type": "response.completed",
                    "response": outgoing_response(ChatCompletionResult {
                        model,
                        message: ChatMessage {
                            role: ChatRole::Assistant,
                            content: output.clone(),
                        },
                        finish_reason,
                        usage,
                    })
                })
                .to_string(),
            ),
        ),
        Err(error) => Ok::<Event, Infallible>(
            Event::default().event("error").data(
                json!({
                    "error": {
                        "message": error.to_string()
                    }
                })
                .to_string(),
            ),
        ),
    });
    let done = stream::once(async { Ok::<Event, Infallible>(Event::default().data("[DONE]")) });

    Sse::new(created.chain(content).chain(done))
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn load_operator_state(state: &DaemonState) -> Result<OperatorStateResponse> {
    let profiles = state.storage.list_profiles()?;
    let keys = state.storage.list_keys()?;
    let models = state.storage.list_models()?;
    let logs = state.storage.list_request_logs(24)?;
    let profile_count = profiles.len();
    let model_count = models.len();
    let key_count = keys.len();
    let log_count = logs.len();

    let profile_rows = profiles
        .iter()
        .map(|profile| OperatorProfileRow {
            id: profile.id,
            provider: profile.provider.to_string(),
            name: profile.name.clone(),
            selector: format!("{}:{}", profile.provider, profile.name),
            base_url: profile.base_url.clone(),
            auth_label: operator_profile_auth_label(profile),
            model_count: models
                .iter()
                .filter(|model| model.profile_id == Some(profile.id))
                .count(),
        })
        .collect::<Vec<_>>();

    let key_rows = keys
        .into_iter()
        .map(|key| OperatorKeyRow {
            id: key.id,
            name: key.name,
            prefix: key.prefix,
            state: key.state.to_string(),
            scopes: key
                .scopes
                .into_iter()
                .map(|scope| scope.to_string())
                .collect(),
            providers: key
                .allowed_providers
                .into_iter()
                .map(|provider| provider.to_string())
                .collect(),
            last_used_at: key.last_used_at.map(|value| value.to_rfc3339()),
        })
        .collect::<Vec<_>>();

    let log_rows = logs
        .iter()
        .map(|log| {
            let profile_name = profile_rows
                .iter()
                .find(|profile| Some(profile.id) == log.profile_id)
                .map(|profile| profile.name.clone());
            OperatorLogRow {
                id: log.id,
                started_at: log.started_at.to_rfc3339(),
                provider: log.provider.to_string(),
                profile_name,
                model: log.model.clone(),
                endpoint: log.endpoint.clone(),
                status_code: log.status_code,
                duration_ms: log.duration_ms,
                input_tokens: log.usage.input_tokens,
                output_tokens: log.usage.output_tokens,
                total_tokens: log.usage.total_tokens,
                error_message: log.error_message.clone(),
            }
        })
        .collect::<Vec<_>>();

    let traffic = OperatorTrafficRow {
        recent_requests: log_count,
        success_count: logs
            .iter()
            .filter(|log| {
                log.status_code.is_some_and(|code| code < 400) && log.error_message.is_none()
            })
            .count(),
        error_count: logs
            .iter()
            .filter(|log| {
                log.status_code.is_some_and(|code| code >= 400) || log.error_message.is_some()
            })
            .count(),
        avg_latency_ms: (!logs.is_empty())
            .then(|| logs.iter().map(|log| log.duration_ms).sum::<u64>() / logs.len() as u64),
        input_tokens: logs
            .iter()
            .map(|log| u64::from(log.usage.input_tokens.unwrap_or_default()))
            .sum(),
        output_tokens: logs
            .iter()
            .map(|log| u64::from(log.usage.output_tokens.unwrap_or_default()))
            .sum(),
        total_tokens: logs
            .iter()
            .map(|log| u64::from(log.usage.total_tokens.unwrap_or_default()))
            .sum(),
        latest_request_at: logs.first().map(|log| log.started_at.to_rfc3339()),
    };

    let setup = OperatorSetupRow {
        provider_ready: profile_count > 0,
        models_ready: model_count > 0,
        key_ready: key_count > 0,
        traffic_ready: log_count > 0,
        next_step: if profile_count == 0 {
            "Connect one provider."
        } else if model_count == 0 {
            "Sync models for the provider you just connected."
        } else if key_count == 0 {
            "Create one Gunmetal key for your apps."
        } else if log_count == 0 {
            "Point one app at the local API and send the first request."
        } else {
            "Traffic is flowing. Review requests and token usage below."
        }
        .to_owned(),
    };

    let provider_rows = builtin_providers()
        .into_iter()
        .map(|provider| OperatorProviderRow {
            kind: provider.kind.to_string(),
            class: match provider.class {
                ProviderClass::Subscription => "subscription",
                ProviderClass::Gateway => "gateway",
                ProviderClass::Direct => "direct",
            },
            priority: provider.priority,
        })
        .collect::<Vec<_>>();

    Ok(OperatorStateResponse {
        service: OperatorServiceRow {
            status: "running",
            version: state.version.clone(),
            home: state.paths.root.display().to_string(),
            api_base_url: "/v1".to_owned(),
            web_url: "/app".to_owned(),
        },
        counts: OperatorCountRow {
            profiles: profile_count,
            models: model_count,
            keys: key_count,
            logs: log_count,
        },
        setup,
        traffic,
        providers: provider_rows,
        profiles: profile_rows,
        models: models.into_iter().map(ModelResponse::from).collect(),
        keys: key_rows,
        logs: log_rows,
    })
}

fn require_profile(state: &DaemonState, id: Uuid) -> Result<ProviderProfile, ApiError> {
    state
        .storage
        .get_profile(id)
        .map_err(internal_api_error)?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "profile_not_found",
                "provider not found".to_owned(),
            )
        })
}

fn require_key(state: &DaemonState, id: Uuid) -> Result<GunmetalKey, ApiError> {
    state
        .storage
        .get_key(id)
        .map_err(internal_api_error)?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "key_not_found",
                "key not found".to_owned(),
            )
        })
}

fn operator_profile_auth_label(profile: &ProviderProfile) -> String {
    if supports_browser_login(&profile.provider) {
        if profile.credentials.is_some() {
            "session saved".to_owned()
        } else {
            "signed out".to_owned()
        }
    } else if profile_has_api_key(profile) {
        "api key saved".to_owned()
    } else {
        "missing api key".to_owned()
    }
}

fn profile_has_api_key(profile: &ProviderProfile) -> bool {
    profile
        .credentials
        .as_ref()
        .and_then(|value| value.get("api_key"))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn operator_profile_credentials(provider: &ProviderKind, api_key: Option<String>) -> Option<Value> {
    let api_key = api_key.and_then(trimmed_or_none);
    if needs_api_key(provider) {
        api_key.map(|value| json!({ "api_key": value }))
    } else {
        None
    }
}

fn supports_browser_login(provider: &ProviderKind) -> bool {
    matches!(provider, ProviderKind::Codex | ProviderKind::Copilot)
}

fn needs_api_key(provider: &ProviderKind) -> bool {
    matches!(
        provider,
        ProviderKind::OpenRouter
            | ProviderKind::Zen
            | ProviderKind::OpenAi
            | ProviderKind::Azure
            | ProviderKind::Nvidia
    )
}

fn operator_default_scopes() -> Vec<KeyScope> {
    vec![KeyScope::Inference, KeyScope::ModelsRead]
}

fn trimmed_or_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: String,
}

#[derive(Debug, Serialize)]
struct OperatorStateResponse {
    service: OperatorServiceRow,
    counts: OperatorCountRow,
    setup: OperatorSetupRow,
    traffic: OperatorTrafficRow,
    providers: Vec<OperatorProviderRow>,
    profiles: Vec<OperatorProfileRow>,
    models: Vec<ModelResponse>,
    keys: Vec<OperatorKeyRow>,
    logs: Vec<OperatorLogRow>,
}

#[derive(Debug, Serialize)]
struct OperatorServiceRow {
    status: &'static str,
    version: String,
    home: String,
    api_base_url: String,
    web_url: String,
}

#[derive(Debug, Serialize)]
struct OperatorCountRow {
    profiles: usize,
    models: usize,
    keys: usize,
    logs: usize,
}

#[derive(Debug, Serialize)]
struct OperatorSetupRow {
    provider_ready: bool,
    models_ready: bool,
    key_ready: bool,
    traffic_ready: bool,
    next_step: String,
}

#[derive(Debug, Serialize)]
struct OperatorTrafficRow {
    recent_requests: usize,
    success_count: usize,
    error_count: usize,
    avg_latency_ms: Option<u64>,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    latest_request_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct OperatorProviderRow {
    kind: String,
    class: &'static str,
    priority: usize,
}

#[derive(Debug, Serialize)]
struct OperatorProfileRow {
    id: Uuid,
    provider: String,
    name: String,
    selector: String,
    base_url: Option<String>,
    auth_label: String,
    model_count: usize,
}

#[derive(Debug, Serialize)]
struct OperatorKeyRow {
    id: Uuid,
    name: String,
    prefix: String,
    state: String,
    scopes: Vec<String>,
    providers: Vec<String>,
    last_used_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct OperatorLogRow {
    id: Uuid,
    started_at: String,
    provider: String,
    profile_name: Option<String>,
    model: String,
    endpoint: String,
    status_code: Option<u16>,
    duration_ms: u64,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    total_tokens: Option<u32>,
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateProfilePayload {
    provider: String,
    name: String,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateKeyPayload {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SetKeyStatePayload {
    state: String,
}

#[derive(Debug, Serialize)]
struct OperatorActionResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret: Option<String>,
}

impl OperatorActionResponse {
    fn message(message: String) -> Self {
        Self {
            message,
            auth_url: None,
            user_code: None,
            secret: None,
        }
    }
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
    family: Option<String>,
    context_window: Option<u32>,
    max_output_tokens: Option<u32>,
    input_modalities: Vec<String>,
    output_modalities: Vec<String>,
    supports_attachments: Option<bool>,
    supports_reasoning: Option<bool>,
    supports_tools: Option<bool>,
}

impl From<gunmetal_core::ModelDescriptor> for ModelResponse {
    fn from(value: gunmetal_core::ModelDescriptor) -> Self {
        let owned_by = value.provider.to_string();
        let metadata = value.metadata.unwrap_or_default();
        Self {
            id: value.id,
            object: "model",
            owned_by: owned_by.clone(),
            provider: owned_by,
            family: metadata.family,
            context_window: metadata.context_window,
            max_output_tokens: metadata.max_output_tokens,
            input_modalities: metadata.input_modalities,
            output_modalities: metadata.output_modalities,
            supports_attachments: metadata.supports_attachments,
            supports_reasoning: metadata.supports_reasoning,
            supports_tools: metadata.supports_tools,
        }
    }
}

#[derive(Debug, Deserialize)]
struct IncomingChatCompletionsRequest {
    model: String,
    messages: Vec<IncomingChatMessage>,
    stream: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
    stop: Option<IncomingStop>,
    metadata: Option<serde_json::Map<String, Value>>,
    provider_options: Option<serde_json::Map<String, Value>>,
    gunmetal: Option<IncomingGunmetalOptions>,
}

#[derive(Debug, Deserialize)]
struct IncomingResponsesRequest {
    model: String,
    instructions: Option<String>,
    input: Option<IncomingResponsesInput>,
    stream: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    max_output_tokens: Option<u32>,
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
    stop: Option<IncomingStop>,
    metadata: Option<serde_json::Map<String, Value>>,
    provider_options: Option<serde_json::Map<String, Value>>,
    gunmetal: Option<IncomingGunmetalOptions>,
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
            options: RequestOptions {
                temperature: self.temperature,
                top_p: self.top_p,
                max_output_tokens: self
                    .max_output_tokens
                    .or(self.max_completion_tokens)
                    .or(self.max_tokens),
                stop: self.stop.map(IncomingStop::into_vec).unwrap_or_default(),
                metadata: self.metadata.unwrap_or_default(),
                provider_options: self.provider_options.unwrap_or_default(),
                mode: self.gunmetal.map(|value| value.mode).unwrap_or_default(),
            },
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
            options: RequestOptions {
                temperature: self.temperature,
                top_p: self.top_p,
                max_output_tokens: self.max_completion_tokens.or(self.max_tokens),
                stop: self.stop.map(IncomingStop::into_vec).unwrap_or_default(),
                metadata: self.metadata.unwrap_or_default(),
                provider_options: self.provider_options.unwrap_or_default(),
                mode: self.gunmetal.map(|value| value.mode).unwrap_or_default(),
            },
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncomingStop {
    Single(String),
    Many(Vec<String>),
}

impl IncomingStop {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

#[derive(Debug, Deserialize)]
struct IncomingGunmetalOptions {
    #[serde(default)]
    mode: RequestMode,
}

#[derive(Debug, Clone)]
struct ValidatedChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: RequestOptions,
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
    output_text: String,
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
    options: RequestOptions,
) -> Result<(GunmetalKey, ProviderProfile, ChatCompletionRequest), ApiError> {
    let key = authorize(state, headers, KeyScope::Inference)?;

    let model = if let Some(model) = state.request_cache.model(&model_id) {
        model
    } else {
        let Some(model) = state
            .storage
            .get_model(&model_id)
            .map_err(internal_api_error)?
        else {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "model_not_found",
                format!(
                    "Model '{}' is not registered in Gunmetal. Run `gunmetal models sync <saved-provider>` or call `/v1/models`.",
                    model_id
                ),
            ));
        };
        state.request_cache.insert_model(model.clone());
        model
    };

    if !key.can_access_provider(&model.provider) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "provider_forbidden",
            format!(
                "Key '{}' cannot access provider '{}'. Use a key scoped to that provider or create a new key.",
                key.name, model.provider
            ),
        ));
    }

    let Some(profile_id) = model.profile_id else {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "profile_missing",
            format!(
                "Model '{}' is not attached to a saved provider. Run `gunmetal models sync <saved-provider>` again.",
                model.id
            ),
        ));
    };

    let profile = if let Some(profile) = state.request_cache.profile(profile_id) {
        profile
    } else {
        let Some(profile) = state
            .storage
            .get_profile(profile_id)
            .map_err(internal_api_error)?
        else {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "profile_missing",
                format!(
                    "Saved provider '{}' does not exist anymore. Recreate it, then run `gunmetal models sync <saved-provider>`.",
                    profile_id
                ),
            ));
        };
        state.request_cache.insert_profile(profile.clone());
        profile
    };

    Ok((
        key,
        profile,
        ChatCompletionRequest {
            model: model.id,
            messages,
            stream,
            options,
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
            state.request_logger.log(NewRequestLogEntry {
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
            state.request_logger.log(NewRequestLogEntry {
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
                format!(
                    "Provider request failed for saved provider '{}'. Check `gunmetal auth status {}` and retry. Upstream said: {}",
                    profile.name, profile.name, error
                ),
            ))
        }
    }
}

async fn invoke_provider_stream(
    state: &DaemonState,
    key: GunmetalKey,
    profile: ProviderProfile,
    request: ChatCompletionRequest,
    endpoint: &'static str,
) -> Result<ProviderEventStream, Response> {
    let started_at = Instant::now();
    let request_model = request.model.clone();

    match state
        .providers
        .stream_chat_completion(&profile, &request)
        .await
    {
        Ok(mut provider_stream) => {
            let request_logger = state.request_logger.clone();
            let key_id = key.id;
            let profile_id = profile.id;
            let provider = profile.provider.clone();
            let endpoint_name = endpoint.to_owned();

            Ok(async_stream::try_stream! {
                let mut logged = false;
                while let Some(item) = provider_stream.next().await {
                    match item {
                        Ok(ProviderStreamEvent::Complete { model, finish_reason, usage }) => {
                            if !logged {
                                logged = true;
                                request_logger.log(NewRequestLogEntry {
                                    key_id: Some(key_id),
                                    profile_id: Some(profile_id),
                                    provider: provider.clone(),
                                    model: request_model.clone(),
                                    endpoint: endpoint_name.clone(),
                                    status_code: Some(StatusCode::OK.as_u16()),
                                    duration_ms: started_at.elapsed().as_millis() as u64,
                                    usage: usage.clone(),
                                    error_message: None,
                                });
                            }

                            yield ProviderStreamEvent::Complete {
                                model,
                                finish_reason,
                                usage,
                            };
                        }
                        Ok(event) => yield event,
                        Err(error) => {
                            if !logged {
                                logged = true;
                                request_logger.log(NewRequestLogEntry {
                                    key_id: Some(key_id),
                                    profile_id: Some(profile_id),
                                    provider: provider.clone(),
                                    model: request_model.clone(),
                                    endpoint: endpoint_name.clone(),
                                    status_code: Some(StatusCode::BAD_GATEWAY.as_u16()),
                                    duration_ms: started_at.elapsed().as_millis() as u64,
                                    usage: TokenUsage {
                                        input_tokens: None,
                                        output_tokens: None,
                                        total_tokens: None,
                                    },
                                    error_message: Some(error.to_string()),
                                });
                            }

                            Err::<(), anyhow::Error>(error)?;
                        }
                    }
                }

                if !logged {
                    request_logger.log(NewRequestLogEntry {
                        key_id: Some(key_id),
                        profile_id: Some(profile_id),
                        provider,
                        model: request_model,
                        endpoint: endpoint_name,
                        status_code: Some(StatusCode::OK.as_u16()),
                        duration_ms: started_at.elapsed().as_millis() as u64,
                        usage: TokenUsage {
                            input_tokens: None,
                            output_tokens: None,
                            total_tokens: None,
                        },
                        error_message: None,
                    });
                }
            }
            .boxed())
        }
        Err(error) => {
            state.request_logger.log(NewRequestLogEntry {
                key_id: Some(key.id),
                profile_id: Some(profile.id),
                provider: profile.provider,
                model: request.model,
                endpoint: endpoint.to_owned(),
                status_code: Some(StatusCode::BAD_GATEWAY.as_u16()),
                duration_ms: started_at.elapsed().as_millis() as u64,
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
                format!(
                    "Provider request failed for saved provider '{}'. Check `gunmetal auth status {}` and retry. Upstream said: {}",
                    profile.name, profile.name, error
                ),
            ))
        }
    }
}

async fn invoke_provider_raw_stream(
    state: &DaemonState,
    key: GunmetalKey,
    profile: ProviderProfile,
    request: ChatCompletionRequest,
    endpoint: &'static str,
) -> Result<ProviderByteStream, Response> {
    let started_at = Instant::now();

    match state
        .providers
        .raw_stream_chat_completion(&profile, &request)
        .await
    {
        Ok(provider_stream) => Ok(provider_stream),
        Err(error) => {
            state.request_logger.log(NewRequestLogEntry {
                key_id: Some(key.id),
                profile_id: Some(profile.id),
                provider: profile.provider,
                model: request.model,
                endpoint: endpoint.to_owned(),
                status_code: Some(StatusCode::BAD_GATEWAY.as_u16()),
                duration_ms: started_at.elapsed().as_millis() as u64,
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
                format!(
                    "Provider request failed for saved provider '{}'. Check `gunmetal auth status {}` and retry. Upstream said: {}",
                    profile.name, profile.name, error
                ),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        response::Response,
    };
    use gunmetal_core::{
        ChatCompletionResult, ChatMessage, ChatRole, KeyScope, KeyState, NewGunmetalKey,
        NewProviderProfile, ProviderAuthState, ProviderAuthStatus, ProviderKind,
        ProviderLoginSession, RequestMode, TokenUsage,
    };
    use gunmetal_sdk::{
        ProviderAdapter, ProviderAuthResult, ProviderChatResult, ProviderClass, ProviderDefinition,
        ProviderHub, ProviderLoginResult, ProviderModelSyncResult, ProviderRegistry,
    };
    use gunmetal_storage::{AppPaths, StorageHandle};
    use serde_json::{Value, json};
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
    async fn browser_app_shell_is_served() {
        let fixture = Fixture::new();
        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .uri("/app")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_text(response).await;
        assert!(body.contains("Gunmetal Web"));
        assert!(body.contains("/app/api/state"));
        assert!(body.contains("Loading local state"));
        assert!(body.contains("setup-grid"));
        assert!(body.contains("traffic-grid"));
        assert!(body.contains("profile-form-helper"));
        assert!(body.contains("playground-form"));
        assert!(body.contains("playground-transcript"));
        assert!(body.contains("request-detail"));
    }

    #[tokio::test]
    async fn operator_state_reports_profiles_keys_models_and_logs() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![ProviderKind::Codex]);
        let key = fixture.storage.authenticate_key(&secret).unwrap().unwrap();
        let profile = fixture.storage.list_profiles().unwrap().pop().unwrap();
        fixture
            .storage
            .log_request(gunmetal_core::NewRequestLogEntry {
                key_id: Some(key.id),
                profile_id: Some(profile.id),
                provider: ProviderKind::Codex,
                model: "codex/gpt-5.4".to_owned(),
                endpoint: "/v1/chat/completions".to_owned(),
                status_code: Some(200),
                duration_ms: 12,
                usage: TokenUsage {
                    input_tokens: Some(2),
                    output_tokens: Some(3),
                    total_tokens: Some(5),
                },
                error_message: None,
            })
            .unwrap();

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .uri("/app/api/state")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["counts"]["profiles"], 1);
        assert_eq!(body["counts"]["models"], 1);
        assert_eq!(body["counts"]["keys"], 1);
        assert_eq!(body["counts"]["logs"], 1);
        assert_eq!(body["setup"]["provider_ready"], true);
        assert_eq!(body["setup"]["models_ready"], true);
        assert_eq!(body["setup"]["key_ready"], true);
        assert_eq!(body["setup"]["traffic_ready"], true);
        assert_eq!(
            body["setup"]["next_step"],
            "Traffic is flowing. Review requests and token usage below."
        );
        assert_eq!(body["service"]["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(body["traffic"]["recent_requests"], 1);
        assert_eq!(body["traffic"]["success_count"], 1);
        assert_eq!(body["traffic"]["error_count"], 0);
        assert_eq!(body["traffic"]["avg_latency_ms"], 12);
        assert_eq!(body["traffic"]["input_tokens"], 2);
        assert_eq!(body["traffic"]["output_tokens"], 3);
        assert_eq!(body["traffic"]["total_tokens"], 5);
        assert!(body["traffic"]["latest_request_at"].is_string());
        assert_eq!(body["profiles"][0]["name"], "default");
        assert_eq!(body["keys"][0]["state"], "active");
        assert_eq!(body["logs"][0]["model"], "codex/gpt-5.4");
        assert_eq!(body["logs"][0]["input_tokens"], 2);
        assert_eq!(body["logs"][0]["output_tokens"], 3);
        assert_eq!(body["logs"][0]["total_tokens"], 5);
    }

    #[tokio::test]
    async fn operator_flow_can_create_profile_auth_and_key() {
        let fixture = Fixture::new();

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/app/api/profiles")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "provider": "codex",
                            "name": "browser"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let profile = fixture.storage.list_profiles().unwrap().pop().unwrap();

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/app/api/profiles/{}/auth", profile.id))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["auth_url"], "https://example.com");

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/app/api/profiles/{}/keys", profile.id))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "name": "browser-key"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert!(body["secret"].as_str().unwrap().starts_with("gm_"));
        let keys = fixture.storage.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert!(keys[0].allowed_providers.is_empty());
    }

    #[tokio::test]
    async fn operator_save_profile_updates_matching_provider_and_name() {
        let fixture = Fixture::new();

        let first = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/app/api/profiles")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "provider": "openai",
                            "name": "browser",
                            "base_url": "https://one.example/v1",
                            "api_key": "first"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let second = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/app/api/profiles")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "provider": "openai",
                            "name": "browser",
                            "base_url": "https://two.example/v1",
                            "api_key": "second"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::OK);

        let profiles = fixture.storage.list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].provider, ProviderKind::OpenAi);
        assert_eq!(profiles[0].name, "browser");
        assert_eq!(
            profiles[0].base_url.as_deref(),
            Some("https://two.example/v1")
        );
        assert_eq!(
            profiles[0]
                .credentials
                .as_ref()
                .and_then(|value| value.get("api_key"))
                .and_then(|value| value.as_str()),
            Some("second")
        );
    }

    #[tokio::test]
    async fn operator_created_key_can_list_models_from_any_provider() {
        let fixture = Fixture::new();
        let codex = fixture
            .storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::Codex,
                name: "codex".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();
        let zen = fixture
            .storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::Zen,
                name: "zen".to_owned(),
                base_url: None,
                enabled: true,
                credentials: Some(json!({ "api_key": "zen_test_key" })),
            })
            .unwrap();
        fixture
            .storage
            .replace_models_for_profile(
                &ProviderKind::Codex,
                Some(codex.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "codex/gpt-5.4".to_owned(),
                    provider: ProviderKind::Codex,
                    profile_id: Some(codex.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "GPT-5.4".to_owned(),
                    metadata: None,
                }],
            )
            .unwrap();
        fixture
            .storage
            .replace_models_for_profile(
                &ProviderKind::Zen,
                Some(zen.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "zen/gpt-5.4".to_owned(),
                    provider: ProviderKind::Zen,
                    profile_id: Some(zen.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "GPT-5.4".to_owned(),
                    metadata: None,
                }],
            )
            .unwrap();

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/app/api/profiles/{}/keys", codex.id))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "name": "shared-key"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        let secret = body["secret"].as_str().unwrap();

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
        let model_ids = body["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["id"].as_str())
            .collect::<Vec<_>>();
        assert_eq!(model_ids.len(), 2);
        assert!(model_ids.contains(&"codex/gpt-5.4"));
        assert!(model_ids.contains(&"zen/gpt-5.4"));
    }

    #[tokio::test]
    async fn operator_can_delete_profile_and_its_models() {
        let fixture = Fixture::new();
        let profile = fixture
            .storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::Zen,
                name: "zen".to_owned(),
                base_url: None,
                enabled: true,
                credentials: Some(json!({ "api_key": "zen_test_key" })),
            })
            .unwrap();
        fixture
            .storage
            .replace_models_for_profile(
                &ProviderKind::Zen,
                Some(profile.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "zen/gpt-5.4".to_owned(),
                    provider: ProviderKind::Zen,
                    profile_id: Some(profile.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "GPT-5.4".to_owned(),
                    metadata: None,
                }],
            )
            .unwrap();

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/app/api/profiles/{}", profile.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["message"], "Deleted provider zen.");
        assert!(fixture.storage.get_profile(profile.id).unwrap().is_none());
        assert!(fixture.storage.list_models().unwrap().is_empty());
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
        let body = to_json(response).await;
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Authorization")
        );

        let response = app(fixture.state())
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .header(header::AUTHORIZATION, "Bearer gm_bad_key")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_json(response).await;
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("gunmetal setup")
        );

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
        let body = to_json(response).await;
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("gunmetal keys create")
        );
    }

    #[tokio::test]
    async fn revoked_key_is_rejected_even_after_it_was_cached() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::ModelsRead], vec![ProviderKind::Codex]);
        let state = fixture.state();

        let first = app(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let key = fixture.storage.authenticate_key(&secret).unwrap().unwrap();
        fixture
            .storage
            .set_key_state(key.id, KeyState::Revoked)
            .unwrap();

        let second = app(state)
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::UNAUTHORIZED);
        let body = to_json(second).await;
        assert_eq!(body["error"]["code"], "invalid_api_key");
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
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("gunmetal models sync")
        );
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

        let logs = wait_for_logs(&fixture.storage, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status_code, Some(200));
    }

    #[tokio::test]
    async fn chat_completion_rejects_provider_mismatch_with_recovery_text() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![ProviderKind::OpenAi]);

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
                            "messages": [{ "role": "user", "content": "ping" }]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = to_json(response).await;
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("create a new key")
        );
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
    async fn chat_completion_preserves_optional_request_fields() {
        let fixture = Fixture::new();
        fixture.seed_models();
        let secret = fixture.create_key(vec![KeyScope::Inference], vec![ProviderKind::Codex]);
        let seen = Arc::new(Mutex::new(None));

        let response = app(fixture.state_with_spy(seen.clone()))
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
                            "temperature": 0.2,
                            "top_p": 0.9,
                            "max_tokens": 128,
                            "stop": "DONE",
                            "metadata": { "suite": "daemon" },
                            "provider_options": { "reasoning": { "effort": "high" } },
                            "gunmetal": { "mode": "passthrough" }
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let request = seen.lock().unwrap().clone().unwrap();
        assert_eq!(request.options.temperature, Some(0.2));
        assert_eq!(request.options.top_p, Some(0.9));
        assert_eq!(request.options.max_output_tokens, Some(128));
        assert_eq!(request.options.stop, vec!["DONE"]);
        assert_eq!(request.options.metadata["suite"], "daemon");
        assert_eq!(
            request.options.provider_options["reasoning"]["effort"],
            "high"
        );
        assert_eq!(request.options.mode, RequestMode::Passthrough);
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
        assert_eq!(body["output_text"], "hello from codex");
        assert_eq!(body["output"][0]["type"], "message");
        assert_eq!(body["output"][0]["role"], "assistant");
        assert_eq!(body["output"][0]["content"][0]["type"], "output_text");
        assert_eq!(body["output"][0]["content"][0]["text"], "hello from codex");

        let logs = wait_for_logs(&fixture.storage, 1);
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
            let mut registry = ProviderRegistry::default();
            registry.register(MockCodexAdapter);
            let providers = ProviderHub::with_registry(self.paths.clone(), registry);
            DaemonState::with_provider_hub(self.paths.clone(), providers).unwrap()
        }

        fn state_with_spy(
            &self,
            seen: Arc<Mutex<Option<gunmetal_core::ChatCompletionRequest>>>,
        ) -> DaemonState {
            let mut registry = ProviderRegistry::default();
            registry.register(SpyCodexAdapter { seen });
            let providers = ProviderHub::with_registry(self.paths.clone(), registry);
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
                        metadata: None,
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

    fn wait_for_logs(
        storage: &StorageHandle,
        expected: usize,
    ) -> Vec<gunmetal_core::RequestLogEntry> {
        for _ in 0..40 {
            let logs = storage.list_request_logs(10).unwrap();
            if logs.len() >= expected {
                return logs;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        storage.list_request_logs(10).unwrap()
    }

    struct MockCodexAdapter;

    #[async_trait]
    impl ProviderAdapter for MockCodexAdapter {
        fn definition(&self) -> ProviderDefinition {
            ProviderDefinition {
                kind: ProviderKind::Codex,
                class: ProviderClass::Subscription,
                priority: 1,
            }
        }

        async fn auth_status(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<ProviderAuthResult> {
            Ok(ProviderAuthResult {
                credentials: None,
                status: ProviderAuthStatus {
                    state: ProviderAuthState::Connected,
                    label: "codex".to_owned(),
                },
            })
        }

        async fn login(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
            _open_browser: bool,
        ) -> anyhow::Result<ProviderLoginResult> {
            Ok(ProviderLoginResult {
                credentials: None,
                session: ProviderLoginSession {
                    login_id: "mock".to_owned(),
                    auth_url: "https://example.com".to_owned(),
                    user_code: None,
                    interval_seconds: None,
                },
            })
        }

        async fn logout(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<Option<Value>> {
            Ok(None)
        }

        async fn sync_models(
            &self,
            profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<ProviderModelSyncResult> {
            Ok(ProviderModelSyncResult {
                credentials: None,
                models: vec![gunmetal_core::ModelDescriptor {
                    id: "codex/gpt-5.4".to_owned(),
                    provider: ProviderKind::Codex,
                    profile_id: Some(profile.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "GPT-5.4".to_owned(),
                    metadata: None,
                }],
            })
        }

        async fn chat_completion(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
            request: &gunmetal_core::ChatCompletionRequest,
        ) -> anyhow::Result<ProviderChatResult> {
            Ok(ProviderChatResult {
                credentials: None,
                completion: ChatCompletionResult {
                    model: request.model.clone(),
                    message: ChatMessage {
                        role: ChatRole::Assistant,
                        content: "hello from codex".to_owned(),
                    },
                    finish_reason: "stop".to_owned(),
                    usage: TokenUsage {
                        input_tokens: Some(5),
                        output_tokens: Some(2),
                        total_tokens: Some(7),
                    },
                },
            })
        }
    }

    struct SpyCodexAdapter {
        seen: Arc<Mutex<Option<gunmetal_core::ChatCompletionRequest>>>,
    }

    #[async_trait]
    impl ProviderAdapter for SpyCodexAdapter {
        fn definition(&self) -> ProviderDefinition {
            ProviderDefinition {
                kind: ProviderKind::Codex,
                class: ProviderClass::Subscription,
                priority: 1,
            }
        }

        async fn auth_status(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<ProviderAuthResult> {
            Ok(ProviderAuthResult {
                credentials: None,
                status: ProviderAuthStatus {
                    state: ProviderAuthState::Connected,
                    label: "codex".to_owned(),
                },
            })
        }

        async fn login(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
            _open_browser: bool,
        ) -> anyhow::Result<ProviderLoginResult> {
            Ok(ProviderLoginResult {
                credentials: None,
                session: ProviderLoginSession {
                    login_id: "mock".to_owned(),
                    auth_url: "https://example.com".to_owned(),
                    user_code: None,
                    interval_seconds: None,
                },
            })
        }

        async fn logout(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<Option<Value>> {
            Ok(None)
        }

        async fn sync_models(
            &self,
            profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<ProviderModelSyncResult> {
            Ok(ProviderModelSyncResult {
                credentials: None,
                models: vec![gunmetal_core::ModelDescriptor {
                    id: "codex/gpt-5.4".to_owned(),
                    provider: ProviderKind::Codex,
                    profile_id: Some(profile.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "GPT-5.4".to_owned(),
                    metadata: None,
                }],
            })
        }

        async fn chat_completion(
            &self,
            _profile: &gunmetal_core::ProviderProfile,
            _paths: &AppPaths,
            request: &gunmetal_core::ChatCompletionRequest,
        ) -> anyhow::Result<ProviderChatResult> {
            *self.seen.lock().unwrap() = Some(request.clone());
            Ok(ProviderChatResult {
                credentials: None,
                completion: ChatCompletionResult {
                    model: request.model.clone(),
                    message: ChatMessage {
                        role: ChatRole::Assistant,
                        content: "hello from codex".to_owned(),
                    },
                    finish_reason: "stop".to_owned(),
                    usage: TokenUsage {
                        input_tokens: Some(5),
                        output_tokens: Some(2),
                        total_tokens: Some(7),
                    },
                },
            })
        }
    }
}
