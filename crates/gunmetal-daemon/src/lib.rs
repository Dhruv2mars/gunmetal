use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use gunmetal_core::{KeyScope, ModelDescriptor, NewRequestLogEntry, TokenUsage};
use gunmetal_storage::StorageHandle;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug)]
pub struct DaemonState {
    pub storage: StorageHandle,
    pub version: String,
}

impl DaemonState {
    pub fn new(storage: StorageHandle) -> Self {
        Self {
            storage,
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

pub fn app(state: DaemonState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
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

impl From<ModelDescriptor> for ModelResponse {
    fn from(value: ModelDescriptor) -> Self {
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
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

impl ChatCompletionsRequest {
    fn validate(self) -> Result<Self, ApiError> {
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

        if self
            .messages
            .iter()
            .any(|message| message.role.trim().is_empty())
        {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "message role cannot be empty".to_owned(),
            ));
        }

        if self
            .messages
            .iter()
            .any(|message| message.content.trim().is_empty())
        {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "message content cannot be empty".to_owned(),
            ));
        }

        Ok(self)
    }
}

async fn chat_completions(
    State(state): State<DaemonState>,
    headers: HeaderMap,
    Json(payload): Json<ChatCompletionsRequest>,
) -> Response {
    let payload = match payload.validate() {
        Ok(payload) => payload,
        Err(error) => return error.into_response(),
    };

    let _stream_requested = payload.stream.unwrap_or(false);

    let key = match authorize(&state, &headers, KeyScope::Inference) {
        Ok(key) => key,
        Err(error) => return error.into_response(),
    };

    let models = match state.storage.list_models() {
        Ok(models) => models,
        Err(error) => return internal_error(error),
    };

    let Some(model) = models.into_iter().find(|item| item.id == payload.model) else {
        return api_error(
            StatusCode::NOT_FOUND,
            "model_not_found",
            format!("model '{}' is not registered in gunmetal", payload.model),
        );
    };

    if !key.can_access_provider(&model.provider) {
        return api_error(
            StatusCode::FORBIDDEN,
            "provider_forbidden",
            format!(
                "key '{}' cannot access provider '{}'",
                key.name, model.provider
            ),
        );
    }

    let _ = state.storage.log_request(NewRequestLogEntry {
        key_id: Some(key.id),
        profile_id: model.profile_id,
        provider: model.provider.clone(),
        model: model.id.clone(),
        endpoint: "/v1/chat/completions".to_owned(),
        status_code: Some(StatusCode::NOT_IMPLEMENTED.as_u16()),
        duration_ms: 0,
        usage: TokenUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
        },
        error_message: Some("provider adapter not wired yet".to_owned()),
    });

    api_error(
        StatusCode::NOT_IMPLEMENTED,
        "provider_not_implemented",
        format!(
            "model '{}' is registered, but provider '{}' is not wired yet",
            model.id, model.provider
        ),
    )
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

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        response::Response,
    };
    use gunmetal_core::{KeyScope, ModelDescriptor, NewGunmetalKey, ProviderKind};
    use gunmetal_storage::StorageHandle;
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
    async fn chat_completion_logs_attempt_for_registered_model() {
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

        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let logs = fixture.storage.list_request_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].model, "codex/gpt-5.4");
    }

    struct Fixture {
        _temp: TempDir,
        storage: StorageHandle,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TempDir::new().unwrap();
            let storage = StorageHandle::new(temp.path().join("gunmetal.db")).unwrap();
            Self {
                _temp: temp,
                storage,
            }
        }

        fn state(&self) -> DaemonState {
            DaemonState::new(self.storage.clone())
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
            self.storage
                .upsert_model_registry(&[
                    ModelDescriptor {
                        id: "codex/gpt-5.4".to_owned(),
                        provider: ProviderKind::Codex,
                        profile_id: None,
                        upstream_name: "gpt-5.4".to_owned(),
                        display_name: "GPT-5.4".to_owned(),
                    },
                    ModelDescriptor {
                        id: "copilot/claude-sonnet-4.5".to_owned(),
                        provider: ProviderKind::Copilot,
                        profile_id: None,
                        upstream_name: "claude-sonnet-4.5".to_owned(),
                        display_name: "Claude Sonnet 4.5".to_owned(),
                    },
                ])
                .unwrap();
        }
    }

    async fn to_json(response: Response) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }
}
