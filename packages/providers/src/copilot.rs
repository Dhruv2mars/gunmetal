use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow, bail};
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ChatMessage, ChatRole, ModelDescriptor,
    ProviderAuthState, ProviderAuthStatus, ProviderKind, ProviderLoginSession, ProviderProfile,
    TokenUsage,
};
use reqwest::{
    Client, Response,
    header::{self, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEFAULT_API_BASE_URL: &str = "https://api.github.com";
const DEFAULT_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";
const DEFAULT_COPILOT_BASE_URL: &str = "https://api.githubcopilot.com";
const DEFAULT_LOGIN_BASE_URL: &str = "https://github.com/login";
const REFRESH_SKEW_SECONDS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopilotSession {
    pub account_label: String,
    pub expires_at: Option<String>,
    pub organization: Option<String>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_at: Option<String>,
    pub token: String,
    pub token_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopilotClientOptions {
    api_base_url: String,
    client_id: String,
    copilot_base_url: String,
    login_base_url: String,
    pending_login: Option<CopilotPendingLogin>,
    scope: Option<String>,
    session: Option<CopilotSession>,
}

impl CopilotClientOptions {
    pub fn from_profile(profile: &ProviderProfile) -> Self {
        let settings = profile
            .credentials
            .clone()
            .map(CopilotCredentialEnvelope::from_value)
            .unwrap_or_default();

        Self {
            api_base_url: settings
                .api_base_url
                .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_owned()),
            client_id: settings
                .client_id
                .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_owned()),
            copilot_base_url: settings
                .copilot_base_url
                .or_else(|| profile.base_url.clone())
                .unwrap_or_else(|| DEFAULT_COPILOT_BASE_URL.to_owned()),
            login_base_url: settings
                .login_base_url
                .unwrap_or_else(|| DEFAULT_LOGIN_BASE_URL.to_owned()),
            pending_login: settings.pending_login,
            scope: settings.scope,
            session: settings.session,
        }
    }

    fn persisted_credentials(
        &self,
        session: Option<CopilotSession>,
        pending_login: Option<CopilotPendingLogin>,
    ) -> Option<Value> {
        let envelope = CopilotCredentialEnvelope {
            api_base_url: (self.api_base_url != DEFAULT_API_BASE_URL)
                .then(|| self.api_base_url.clone()),
            client_id: (self.client_id != DEFAULT_CLIENT_ID).then(|| self.client_id.clone()),
            copilot_base_url: (self.copilot_base_url != DEFAULT_COPILOT_BASE_URL)
                .then(|| self.copilot_base_url.clone()),
            login_base_url: (self.login_base_url != DEFAULT_LOGIN_BASE_URL)
                .then(|| self.login_base_url.clone()),
            pending_login,
            scope: self.scope.clone(),
            session,
        };

        envelope.into_value()
    }
}

#[derive(Debug, Clone)]
pub struct CopilotAuthStatusResult {
    pub credentials: Option<Value>,
    pub status: ProviderAuthStatus,
}

#[derive(Debug, Clone)]
pub struct CopilotLoginResult {
    pub credentials: Option<Value>,
    pub session: ProviderLoginSession,
}

#[derive(Debug, Clone)]
pub struct CopilotModelSyncResult {
    pub credentials: Option<Value>,
    pub models: Vec<ModelDescriptor>,
}

#[derive(Debug, Clone)]
pub struct CopilotChatResult {
    pub completion: ChatCompletionResult,
    pub credentials: Option<Value>,
}

#[derive(Clone)]
pub struct CopilotClient {
    http: Client,
    model_cache: Arc<Mutex<HashMap<String, LiveModel>>>,
    mode: CopilotMode,
}

#[derive(Clone)]
enum CopilotMode {
    Live(Box<CopilotClientOptions>),
    Mock(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct CopilotCredentialEnvelope {
    #[serde(default)]
    api_base_url: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    copilot_base_url: Option<String>,
    #[serde(default)]
    login_base_url: Option<String>,
    #[serde(default)]
    pending_login: Option<CopilotPendingLogin>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    session: Option<CopilotSession>,
}

impl CopilotCredentialEnvelope {
    fn from_value(value: Value) -> Self {
        let parsed = serde_json::from_value::<Self>(value.clone()).unwrap_or_default();
        let looks_empty = parsed.api_base_url.is_none()
            && parsed.client_id.is_none()
            && parsed.copilot_base_url.is_none()
            && parsed.login_base_url.is_none()
            && parsed.pending_login.is_none()
            && parsed.scope.is_none()
            && parsed.session.is_none();
        if !looks_empty {
            return parsed;
        }

        serde_json::from_value::<CopilotSession>(value)
            .map(|session| Self {
                session: Some(session),
                ..Self::default()
            })
            .unwrap_or_default()
    }

    fn into_value(self) -> Option<Value> {
        let is_empty = self.api_base_url.is_none()
            && self.client_id.is_none()
            && self.copilot_base_url.is_none()
            && self.login_base_url.is_none()
            && self.pending_login.is_none()
            && self.scope.is_none()
            && self.session.is_none();
        if is_empty {
            None
        } else {
            Some(serde_json::to_value(self).expect("serialize copilot credentials"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CopilotPendingLogin {
    device_code: String,
    expires_at: String,
    interval_seconds: u64,
    organization: Option<String>,
    user_code: String,
    verification_uri: String,
}

impl CopilotPendingLogin {
    fn active(&self) -> bool {
        self.expires_at
            .parse::<u64>()
            .ok()
            .is_some_and(|value| value > epoch_seconds())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeviceAuthorizationStatus {
    Complete(CopilotSession),
    Pending { interval_seconds: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct AccessTokenPayload {
    access_token: Option<String>,
    error: Option<String>,
    expires_in: Option<u64>,
    interval: Option<u64>,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DeviceCodePayload {
    device_code: Option<String>,
    expires_in: Option<u64>,
    interval: Option<u64>,
    user_code: Option<String>,
    verification_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ModelEnvelope {
    data: Vec<RemoteModelRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct RemoteCapabilities {
    family: Option<String>,
    #[serde(rename = "type")]
    model_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RemoteModelPolicy {
    state: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RemoteModelRecord {
    capabilities: Option<RemoteCapabilities>,
    id: String,
    model_picker_enabled: Option<bool>,
    name: Option<String>,
    policy: Option<RemoteModelPolicy>,
    preview: Option<bool>,
    supported_endpoints: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiveModel {
    family: Option<String>,
    id: String,
    label: String,
    model_picker_enabled: bool,
    policy_state: Option<String>,
    preview: bool,
    supported_endpoints: Vec<String>,
    model_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionChunk {
    choices: Option<Vec<ChatCompletionChoice>>,
    usage: Option<ChatUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionChoice {
    message: Option<ChatDelta>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatDelta {
    content: Option<ChatContent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ChatContent {
    Parts(Vec<TextPart>),
    Text(String),
}

#[derive(Debug, Clone, Deserialize)]
struct ChatUsage {
    completion_tokens: Option<u64>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    prompt_tokens: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResponsesPayload {
    output: Option<Vec<ResponsesItem>>,
    usage: Option<ResponsesUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResponsesItem {
    content: Option<Vec<ResponsesContentItem>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResponsesContentItem {
    text: Option<String>,
    #[serde(rename = "type")]
    part_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResponsesUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct TextPart {
    text: Option<String>,
    #[serde(rename = "type")]
    part_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EndpointKind {
    Chat,
    Responses,
}

#[derive(Debug, Clone)]
struct UpstreamError {
    code: Option<String>,
    message: String,
    _status: u16,
}

impl UpstreamError {
    fn new(status: u16, code: Option<String>, message: &str) -> Self {
        Self {
            code,
            message: message.to_owned(),
            _status: status,
        }
    }
}

impl std::fmt::Display for UpstreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for UpstreamError {}

impl CopilotClient {
    pub fn with_options(options: CopilotClientOptions) -> Self {
        Self {
            http: Client::builder().build().expect("reqwest client"),
            model_cache: Arc::new(Mutex::new(HashMap::new())),
            mode: CopilotMode::Live(Box::new(options)),
        }
    }

    pub fn mock(response: impl Into<String>) -> Self {
        Self {
            http: Client::builder().build().expect("reqwest client"),
            model_cache: Arc::new(Mutex::new(HashMap::new())),
            mode: CopilotMode::Mock(response.into()),
        }
    }

    pub fn is_mock(&self) -> bool {
        matches!(self.mode, CopilotMode::Mock(_))
    }

    pub async fn auth_status(&self, profile: &ProviderProfile) -> Result<CopilotAuthStatusResult> {
        match &self.mode {
            CopilotMode::Mock(_) => Ok(CopilotAuthStatusResult {
                credentials: profile.credentials.clone(),
                status: ProviderAuthStatus {
                    state: ProviderAuthState::Connected,
                    label: "mock@gunmetal (copilot)".to_owned(),
                },
            }),
            CopilotMode::Live(options) => {
                if let Some(pending) = &options.pending_login {
                    if !pending.active() {
                        return Ok(CopilotAuthStatusResult {
                            credentials: options.persisted_credentials(None, None),
                            status: ProviderAuthStatus {
                                state: ProviderAuthState::Expired,
                                label: "GitHub Copilot login expired".to_owned(),
                            },
                        });
                    }

                    match self
                        .poll_device_authorization(&options.client_id, pending)
                        .await?
                    {
                        DeviceAuthorizationStatus::Complete(session) => {
                            Ok(CopilotAuthStatusResult {
                                credentials: options
                                    .persisted_credentials(Some(session.clone()), None),
                                status: ProviderAuthStatus {
                                    state: ProviderAuthState::Connected,
                                    label: session.account_label,
                                },
                            })
                        }
                        DeviceAuthorizationStatus::Pending { interval_seconds } => {
                            let mut pending = pending.clone();
                            pending.interval_seconds = interval_seconds;
                            Ok(CopilotAuthStatusResult {
                                credentials: options.persisted_credentials(None, Some(pending)),
                                status: ProviderAuthStatus {
                                    state: ProviderAuthState::SigningIn,
                                    label: "Waiting for GitHub Copilot authorization".to_owned(),
                                },
                            })
                        }
                    }
                } else if let Some(session) = options.session.clone() {
                    let session = self.refresh_if_needed(options, session).await?;
                    Ok(CopilotAuthStatusResult {
                        credentials: options.persisted_credentials(Some(session.clone()), None),
                        status: ProviderAuthStatus {
                            state: ProviderAuthState::Connected,
                            label: session.account_label,
                        },
                    })
                } else {
                    Ok(CopilotAuthStatusResult {
                        credentials: options.persisted_credentials(None, None),
                        status: ProviderAuthStatus {
                            state: ProviderAuthState::SignedOut,
                            label: "Signed out".to_owned(),
                        },
                    })
                }
            }
        }
    }

    pub async fn login(
        &self,
        profile: &ProviderProfile,
        open_browser: bool,
    ) -> Result<CopilotLoginResult> {
        match &self.mode {
            CopilotMode::Mock(_) => Ok(CopilotLoginResult {
                credentials: profile.credentials.clone(),
                session: ProviderLoginSession {
                    login_id: "mock-device-code".to_owned(),
                    auth_url: "https://github.com/login/device".to_owned(),
                    user_code: Some("ABCD-EFGH".to_owned()),
                    interval_seconds: Some(5),
                },
            }),
            CopilotMode::Live(options) => {
                let pending = self.start_device_authorization(options).await?;
                if open_browser {
                    let _ = webbrowser::open(&pending.verification_uri);
                }
                Ok(CopilotLoginResult {
                    credentials: options.persisted_credentials(None, Some(pending.clone())),
                    session: ProviderLoginSession {
                        login_id: pending.device_code.clone(),
                        auth_url: pending.verification_uri.clone(),
                        user_code: Some(pending.user_code),
                        interval_seconds: Some(pending.interval_seconds),
                    },
                })
            }
        }
    }

    pub async fn list_models(&self, profile: &ProviderProfile) -> Result<CopilotModelSyncResult> {
        match &self.mode {
            CopilotMode::Mock(_) => Ok(CopilotModelSyncResult {
                credentials: profile.credentials.clone(),
                models: vec![ModelDescriptor {
                    id: "copilot/gpt-5.4".to_owned(),
                    provider: ProviderKind::Copilot,
                    profile_id: Some(profile.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "GPT-5.4".to_owned(),
                }],
            }),
            CopilotMode::Live(options) => {
                let session = self.ensure_session(options).await?;
                let live = self.fetch_model_records(options, &session.token).await?;
                let mut deduped = HashMap::new();
                for model in live
                    .into_iter()
                    .filter(|model| model.policy_state.as_deref() != Some("disabled"))
                    .filter(|model| model.model_type.as_deref() != Some("embedding"))
                    .filter(|model| model.model_picker_enabled || model.model_type.is_some())
                {
                    let score = score_model(&model);
                    let upstream_name = model.id.clone();
                    let descriptor = ModelDescriptor {
                        id: format!("copilot/{upstream_name}"),
                        provider: ProviderKind::Copilot,
                        profile_id: Some(profile.id),
                        upstream_name,
                        display_name: model.label.clone(),
                    };
                    match deduped.get(&descriptor.id) {
                        Some((current_score, _)) if *current_score >= score => {}
                        _ => {
                            deduped.insert(descriptor.id.clone(), (score, descriptor));
                        }
                    }
                }
                let mut models = deduped
                    .into_values()
                    .map(|(_, descriptor)| descriptor)
                    .collect::<Vec<_>>();
                models.sort_by(|left, right| left.id.cmp(&right.id));

                Ok(CopilotModelSyncResult {
                    credentials: options.persisted_credentials(Some(session), None),
                    models,
                })
            }
        }
    }

    pub async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<CopilotChatResult> {
        match &self.mode {
            CopilotMode::Mock(response) => Ok(CopilotChatResult {
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
            CopilotMode::Live(options) => {
                let session = self.ensure_session(options).await?;
                let model = request
                    .model
                    .strip_prefix("copilot/")
                    .unwrap_or(&request.model)
                    .to_owned();

                if !self.has_model_cached(&model) {
                    let _ = self.fetch_model_records(options, &session.token).await?;
                }

                let completion = self
                    .complete_with_fallback(options, &session.token, &model, &request.messages)
                    .await?;

                Ok(CopilotChatResult {
                    credentials: options.persisted_credentials(Some(session), None),
                    completion,
                })
            }
        }
    }

    async fn ensure_session(&self, options: &CopilotClientOptions) -> Result<CopilotSession> {
        if let Some(pending) = &options.pending_login {
            match self
                .poll_device_authorization(&options.client_id, pending)
                .await?
            {
                DeviceAuthorizationStatus::Complete(session) => return Ok(session),
                DeviceAuthorizationStatus::Pending { .. } => {
                    bail!("copilot authorization pending");
                }
            }
        }

        let Some(session) = options.session.clone() else {
            bail!("copilot not authenticated");
        };

        self.refresh_if_needed(options, session).await
    }

    async fn refresh_if_needed(
        &self,
        options: &CopilotClientOptions,
        session: CopilotSession,
    ) -> Result<CopilotSession> {
        if !session_needs_refresh(&session) {
            return Ok(session);
        }

        let refresh_token = session
            .refresh_token
            .as_deref()
            .ok_or_else(|| anyhow!("copilot refresh token unavailable"))?;
        let payload = self
            .request_access_token(
                options,
                &[
                    ("client_id", options.client_id.as_str()),
                    ("grant_type", "refresh_token"),
                    ("refresh_token", refresh_token),
                ],
            )
            .await?;
        let access_token = payload.access_token.ok_or_else(|| {
            anyhow!(
                payload
                    .error
                    .unwrap_or_else(|| "copilot refresh failed".to_owned())
            )
        })?;

        self.connect(
            options,
            &access_token,
            session.organization.as_deref(),
            payload.refresh_token.or(session.refresh_token),
            payload.refresh_token_expires_in.or_else(|| {
                session
                    .refresh_token_expires_at
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
            }),
            payload.expires_in,
        )
        .await
    }

    async fn start_device_authorization(
        &self,
        options: &CopilotClientOptions,
    ) -> Result<CopilotPendingLogin> {
        let mut form = vec![("client_id", options.client_id.as_str())];
        if let Some(scope) = options.scope.as_deref() {
            form.push(("scope", scope));
        }
        let response = self
            .http
            .post(format!("{}/device/code", options.login_base_url))
            .header(header::ACCEPT, "application/json")
            .form(&form)
            .send()
            .await?;
        let status = response.status();
        let payload: DeviceCodePayload = response.json().await?;
        if !status.is_success() {
            bail!("github device code request failed");
        }

        Ok(CopilotPendingLogin {
            device_code: payload
                .device_code
                .ok_or_else(|| anyhow!("github device code missing"))?,
            expires_at: seconds_from_now(payload.expires_in.unwrap_or(900)),
            interval_seconds: payload.interval.unwrap_or(5),
            organization: None,
            user_code: payload
                .user_code
                .ok_or_else(|| anyhow!("github user code missing"))?,
            verification_uri: payload
                .verification_uri
                .ok_or_else(|| anyhow!("github verification uri missing"))?,
        })
    }

    async fn poll_device_authorization(
        &self,
        client_id: &str,
        pending: &CopilotPendingLogin,
    ) -> Result<DeviceAuthorizationStatus> {
        let payload = self
            .request_access_token(
                &self.options()?,
                &[
                    ("client_id", client_id),
                    ("device_code", pending.device_code.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ],
            )
            .await?;

        if matches!(
            payload.error.as_deref(),
            Some("authorization_pending") | Some("slow_down")
        ) {
            return Ok(DeviceAuthorizationStatus::Pending {
                interval_seconds: payload.interval.unwrap_or(pending.interval_seconds),
            });
        }

        let access_token = payload.access_token.ok_or_else(|| {
            anyhow!(
                payload
                    .error
                    .unwrap_or_else(|| "github auth failed".to_owned())
            )
        })?;

        Ok(DeviceAuthorizationStatus::Complete(
            self.connect(
                &self.options()?,
                &access_token,
                pending.organization.as_deref(),
                payload.refresh_token,
                payload.refresh_token_expires_in,
                payload.expires_in,
            )
            .await?,
        ))
    }

    async fn connect(
        &self,
        options: &CopilotClientOptions,
        token: &str,
        organization: Option<&str>,
        refresh_token: Option<String>,
        refresh_token_expires_in: Option<u64>,
        expires_in: Option<u64>,
    ) -> Result<CopilotSession> {
        let _ = self.fetch_model_records(options, token).await?;
        let account_label = self
            .lookup_account_label(options, token)
            .await
            .unwrap_or_else(|_| "GitHub Copilot".to_owned());

        Ok(CopilotSession {
            account_label,
            expires_at: expires_in.map(seconds_from_now),
            organization: organization.map(str::to_owned),
            refresh_token,
            refresh_token_expires_at: refresh_token_expires_in.map(seconds_from_now),
            token: token.to_owned(),
            token_hint: mask_token(token),
        })
    }

    async fn lookup_account_label(
        &self,
        options: &CopilotClientOptions,
        token: &str,
    ) -> Result<String> {
        let response = self
            .http
            .get(format!("{}/user", options.api_base_url))
            .headers(github_headers(token))
            .send()
            .await?;
        let text = response.text().await?;
        Ok(parse_github_user_body(&text)?.login)
    }

    async fn fetch_model_records(
        &self,
        options: &CopilotClientOptions,
        token: &str,
    ) -> Result<Vec<LiveModel>> {
        let response = self
            .http
            .get(format!("{}/models", options.copilot_base_url))
            .headers(copilot_headers(token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(read_error(response).await.into());
        }

        let text = response.text().await?;
        let records =
            if let Ok(array_payload) = serde_json::from_str::<Vec<RemoteModelRecord>>(&text) {
                array_payload
            } else {
                serde_json::from_str::<ModelEnvelope>(&text)
                    .map(|payload| payload.data)
                    .unwrap_or_default()
            };

        let live = records
            .into_iter()
            .map(|model| LiveModel {
                family: model
                    .capabilities
                    .as_ref()
                    .and_then(|caps| caps.family.clone()),
                id: model.id.clone(),
                label: model.name.unwrap_or_else(|| model.id.clone()),
                model_picker_enabled: model.model_picker_enabled.unwrap_or(false),
                policy_state: model.policy.and_then(|policy| policy.state),
                preview: model.preview.unwrap_or(false),
                supported_endpoints: model.supported_endpoints.unwrap_or_default(),
                model_type: model.capabilities.and_then(|caps| caps.model_type),
            })
            .collect::<Vec<_>>();

        if let Ok(mut cache) = self.model_cache.lock() {
            for model in &live {
                cache.insert(normalize_model_id(model), model.clone());
                cache.insert(model.id.clone(), model.clone());
            }
        }

        Ok(live)
    }

    async fn complete_with_fallback(
        &self,
        options: &CopilotClientOptions,
        token: &str,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatCompletionResult> {
        let order = self.endpoint_order(model);
        let mut last_error: Option<UpstreamError> = None;

        for endpoint in order {
            let result = match endpoint {
                EndpointKind::Chat => {
                    self.complete_chat_completions(options, token, model, messages)
                        .await
                }
                EndpointKind::Responses => {
                    self.complete_responses(options, token, model, messages)
                        .await
                }
            };

            match result {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let try_next = matches!(endpoint, EndpointKind::Chat)
                        && (error.code.as_deref() == Some("unsupported_api_for_model")
                            || error.code.as_deref() == Some("model_not_supported"));
                    last_error = Some(error.clone());
                    if !try_next {
                        return Err(error.into());
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| UpstreamError::new(0, None, "github copilot request failed"))
            .into())
    }

    async fn complete_chat_completions(
        &self,
        options: &CopilotClientOptions,
        token: &str,
        model: &str,
        messages: &[ChatMessage],
    ) -> std::result::Result<ChatCompletionResult, UpstreamError> {
        let response = self
            .http
            .post(format!("{}/chat/completions", options.copilot_base_url))
            .headers(copilot_headers(token))
            .json(&json!({
                "messages": messages.iter().map(to_upstream_message).collect::<Vec<_>>(),
                "model": model,
                "stream": false
            }))
            .send()
            .await
            .map_err(|_| UpstreamError::new(0, None, "github copilot request failed"))?;

        if !response.status().is_success() {
            return Err(read_error(response).await);
        }

        let payload: ChatCompletionChunk = response
            .json()
            .await
            .map_err(|_| UpstreamError::new(0, None, "github copilot request failed"))?;
        let content = payload
            .choices
            .and_then(|choices| choices.into_iter().next())
            .and_then(|choice| choice.message)
            .and_then(|message| message.content)
            .map(read_chat_content)
            .unwrap_or_default();
        let input_tokens = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.prompt_tokens.or(usage.input_tokens))
            .map(to_u32);
        let output_tokens = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.completion_tokens.or(usage.output_tokens))
            .map(to_u32);

        Ok(ChatCompletionResult {
            model: format!("copilot/{model}"),
            message: ChatMessage {
                role: ChatRole::Assistant,
                content,
            },
            finish_reason: "stop".to_owned(),
            usage: usage_from_parts(input_tokens, output_tokens),
        })
    }

    async fn complete_responses(
        &self,
        options: &CopilotClientOptions,
        token: &str,
        model: &str,
        messages: &[ChatMessage],
    ) -> std::result::Result<ChatCompletionResult, UpstreamError> {
        let response = self
            .http
            .post(format!("{}/responses", options.copilot_base_url))
            .headers(copilot_headers(token))
            .json(&json!({
                "input": messages.iter().map(to_responses_message).collect::<Vec<_>>(),
                "model": model,
                "stream": false
            }))
            .send()
            .await
            .map_err(|_| UpstreamError::new(0, None, "github copilot request failed"))?;

        if !response.status().is_success() {
            return Err(read_error(response).await);
        }

        let payload: ResponsesPayload = response
            .json()
            .await
            .map_err(|_| UpstreamError::new(0, None, "github copilot request failed"))?;
        let content = payload
            .output
            .unwrap_or_default()
            .into_iter()
            .flat_map(|item| item.content.unwrap_or_default())
            .filter_map(|part| {
                if matches!(part.part_type.as_deref(), None | Some("output_text")) {
                    part.text
                } else {
                    None
                }
            })
            .collect::<String>();
        let input_tokens = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.input_tokens)
            .map(to_u32);
        let output_tokens = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.output_tokens)
            .map(to_u32);

        Ok(ChatCompletionResult {
            model: format!("copilot/{model}"),
            message: ChatMessage {
                role: ChatRole::Assistant,
                content,
            },
            finish_reason: "stop".to_owned(),
            usage: usage_from_parts(input_tokens, output_tokens),
        })
    }

    async fn request_access_token(
        &self,
        options: &CopilotClientOptions,
        form: &[(&str, &str)],
    ) -> Result<AccessTokenPayload> {
        let response = self
            .http
            .post(format!("{}/oauth/access_token", options.login_base_url))
            .header(header::ACCEPT, "application/json")
            .form(form)
            .send()
            .await?;
        let body = response.text().await?;
        parse_access_token_payload(&body)
    }

    fn endpoint_order(&self, model: &str) -> Vec<EndpointKind> {
        let supported = self
            .model_cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(model).cloned())
            .map(|model| model.supported_endpoints)
            .unwrap_or_default();

        if supported.is_empty() {
            return vec![EndpointKind::Chat, EndpointKind::Responses];
        }
        if supported.iter().any(|endpoint| endpoint == "/responses")
            && !supported
                .iter()
                .any(|endpoint| endpoint == "/chat/completions")
        {
            return vec![EndpointKind::Responses];
        }
        if supported
            .iter()
            .any(|endpoint| endpoint == "/chat/completions")
            && !supported.iter().any(|endpoint| endpoint == "/responses")
        {
            return vec![EndpointKind::Chat];
        }

        vec![EndpointKind::Chat, EndpointKind::Responses]
    }

    fn has_model_cached(&self, model: &str) -> bool {
        self.model_cache
            .lock()
            .ok()
            .is_some_and(|cache| cache.contains_key(model))
    }

    fn options(&self) -> Result<CopilotClientOptions> {
        match &self.mode {
            CopilotMode::Live(options) => Ok((**options).clone()),
            CopilotMode::Mock(_) => bail!("copilot mock has no live options"),
        }
    }
}

async fn read_error(response: Response) -> UpstreamError {
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    let payload = serde_json::from_str::<Value>(&text).ok();
    let code = payload
        .as_ref()
        .and_then(|value| value.get("error"))
        .and_then(|value| value.get("code"))
        .and_then(Value::as_str)
        .map(str::to_owned);
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
        .unwrap_or("github copilot request failed");

    UpstreamError::new(status, code, message)
}

fn parse_access_token_payload(body: &str) -> Result<AccessTokenPayload> {
    if let Ok(payload) = serde_json::from_str::<AccessTokenPayload>(body) {
        return Ok(payload);
    }

    let params = url::form_urlencoded::parse(body.trim().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();
    if params.is_empty() {
        bail!("github auth failed");
    }

    Ok(AccessTokenPayload {
        access_token: params.get("access_token").cloned(),
        error: params.get("error").cloned(),
        expires_in: params
            .get("expires_in")
            .and_then(|value| value.parse::<u64>().ok()),
        interval: params
            .get("interval")
            .and_then(|value| value.parse::<u64>().ok()),
        refresh_token: params.get("refresh_token").cloned(),
        refresh_token_expires_in: params
            .get("refresh_token_expires_in")
            .and_then(|value| value.parse::<u64>().ok()),
    })
}

fn parse_github_user_body(body: &str) -> Result<GitHubUser> {
    let trimmed = body.trim();
    if let Ok(user) = serde_json::from_str::<GitHubUser>(trimmed) {
        return Ok(user);
    }

    for line in trimmed.lines() {
        let candidate = line.trim();
        if candidate.is_empty() {
            continue;
        }
        if let Ok(user) = serde_json::from_str::<GitHubUser>(candidate) {
            return Ok(user);
        }
    }

    bail!("github user lookup failed")
}

fn copilot_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).expect("auth header"),
    );
    headers.insert(
        header::HeaderName::from_static("copilot-integration-id"),
        HeaderValue::from_static("vscode-chat"),
    );
    headers.insert(
        header::HeaderName::from_static("editor-plugin-version"),
        HeaderValue::from_static("copilot-chat/0.30.0"),
    );
    headers.insert(
        header::HeaderName::from_static("editor-version"),
        HeaderValue::from_static("vscode/1.106.0"),
    );
    headers
}

fn github_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).expect("auth header"),
    );
    headers.insert(
        header::HeaderName::from_static("x-github-api-version"),
        HeaderValue::from_static("2022-11-28"),
    );
    headers
}

fn to_upstream_message(message: &ChatMessage) -> Value {
    json!({
        "role": role_name(&message.role),
        "content": message.content
    })
}

fn to_responses_message(message: &ChatMessage) -> Value {
    json!({
        "role": role_name(&message.role),
        "content": [{
            "type": if matches!(message.role, ChatRole::Assistant) {
                "output_text"
            } else {
                "input_text"
            },
            "text": message.content
        }]
    })
}

fn role_name(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::Assistant => "assistant",
        ChatRole::System => "system",
        ChatRole::User => "user",
    }
}

fn read_chat_content(content: ChatContent) -> String {
    match content {
        ChatContent::Text(text) => text,
        ChatContent::Parts(parts) => parts
            .into_iter()
            .filter_map(|part| {
                if matches!(part.part_type.as_deref(), None | Some("text")) {
                    part.text
                } else {
                    None
                }
            })
            .collect::<String>(),
    }
}

fn normalize_model_id(model: &LiveModel) -> String {
    model.family.clone().unwrap_or_else(|| model.id.clone())
}

fn score_model(model: &LiveModel) -> u8 {
    let mut score = 0;
    if model.model_picker_enabled {
        score += 2;
    }
    if !model.preview {
        score += 1;
    }
    if model.family.as_deref() == Some(model.id.as_str()) {
        score += 1;
    }
    score
}

fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        token.to_owned()
    } else {
        format!("{}...{}", &token[..4], &token[token.len() - 4..])
    }
}

fn session_needs_refresh(session: &CopilotSession) -> bool {
    let Some(expires_at) = session.expires_at.as_deref() else {
        return false;
    };
    let Ok(expires_at) = expires_at.parse::<u64>() else {
        return false;
    };
    expires_at <= epoch_seconds() + REFRESH_SKEW_SECONDS
}

fn seconds_from_now(seconds: u64) -> String {
    (epoch_seconds() + seconds).to_string()
}

fn epoch_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time after epoch")
        .as_secs()
}

fn to_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn usage_from_parts(input_tokens: Option<u32>, output_tokens: Option<u32>) -> TokenUsage {
    TokenUsage {
        input_tokens,
        output_tokens,
        total_tokens: match (input_tokens, output_tokens) {
            (Some(input), Some(output)) => Some(input.saturating_add(output)),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use gunmetal_core::{ChatRole, ProviderKind};
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, method, path},
    };

    use super::{
        CopilotClient, CopilotClientOptions, CopilotCredentialEnvelope, CopilotSession,
        parse_access_token_payload, parse_github_user_body,
    };

    #[test]
    fn parses_json_and_form_access_token_payloads() {
        let json_payload = parse_access_token_payload(
            r#"{"access_token":"token-1","refresh_token":"refresh-1","expires_in":3600}"#,
        )
        .unwrap();
        assert_eq!(json_payload.access_token.as_deref(), Some("token-1"));
        assert_eq!(json_payload.refresh_token.as_deref(), Some("refresh-1"));
        assert_eq!(json_payload.expires_in, Some(3600));

        let form_payload = parse_access_token_payload(
            "access_token=token-2&refresh_token=refresh-2&expires_in=1800&interval=5",
        )
        .unwrap();
        assert_eq!(form_payload.access_token.as_deref(), Some("token-2"));
        assert_eq!(form_payload.refresh_token.as_deref(), Some("refresh-2"));
        assert_eq!(form_payload.interval, Some(5));
    }

    #[test]
    fn parses_raw_session_credentials() {
        let envelope = CopilotCredentialEnvelope::from_value(json!({
            "account_label": "GitHub Copilot",
            "expires_at": null,
            "organization": null,
            "refresh_token": null,
            "refresh_token_expires_at": null,
            "token": "ghu_1234",
            "token_hint": "ghu_...1234"
        }));

        assert_eq!(
            envelope.session,
            Some(CopilotSession {
                account_label: "GitHub Copilot".to_owned(),
                expires_at: None,
                organization: None,
                refresh_token: None,
                refresh_token_expires_at: None,
                token: "ghu_1234".to_owned(),
                token_hint: "ghu_...1234".to_owned(),
            })
        );
    }

    #[test]
    fn parses_github_user_from_clean_or_multiline_json() {
        let clean = parse_github_user_body(r#"{"login":"dhruv2mars"}"#).unwrap();
        assert_eq!(clean.login, "dhruv2mars");

        let multiline = parse_github_user_body("{\"login\":\"dhruv2mars\"}\n\nignored").unwrap();
        assert_eq!(multiline.login, "dhruv2mars");
    }

    #[tokio::test]
    async fn starts_device_auth_lists_models_and_completes_chat() {
        let github = MockServer::start().await;
        let copilot = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/device/code"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "device_code": "dev-code",
                "expires_in": 900,
                "interval": 5,
                "user_code": "ABCD-EFGH",
                "verification_uri": "https://github.com/login/device"
            })))
            .mount(&github)
            .await;
        Mock::given(method("POST"))
            .and(path("/oauth/access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "ghu_1234567890",
                "expires_in": 3600,
                "refresh_token": "refresh-token",
                "refresh_token_expires_in": 7200
            })))
            .mount(&github)
            .await;
        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "login": "dhruv2mars"
            })))
            .mount(&github)
            .await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
                "capabilities": { "family": "gpt-5.4", "type": "chat" },
                "id": "gpt-5.4",
                "name": "GPT-5.4",
                "model_picker_enabled": true,
                "policy": { "state": "enabled" },
                "supported_endpoints": ["/chat/completions"]
            }])))
            .mount(&copilot)
            .await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(body_string_contains("gpt-5.4"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices": [{ "message": { "content": "GUNMETAL_COPILOT_OK" } }],
                "usage": { "prompt_tokens": 3, "completion_tokens": 2 }
            })))
            .mount(&copilot)
            .await;

        let client = CopilotClient::with_options(CopilotClientOptions {
            api_base_url: github.uri(),
            client_id: "client-id".to_owned(),
            copilot_base_url: copilot.uri(),
            login_base_url: github.uri(),
            pending_login: None,
            scope: None,
            session: None,
        });

        let profile = gunmetal_core::ProviderProfile {
            id: uuid::Uuid::new_v4(),
            provider: ProviderKind::Copilot,
            name: "copilot".to_owned(),
            base_url: None,
            enabled: true,
            credentials: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let login = client.login(&profile, false).await.unwrap();
        assert_eq!(login.session.user_code.as_deref(), Some("ABCD-EFGH"));

        let authed_client = CopilotClient::with_options(CopilotClientOptions {
            api_base_url: github.uri(),
            client_id: "client-id".to_owned(),
            copilot_base_url: copilot.uri(),
            login_base_url: github.uri(),
            pending_login: serde_json::from_value::<super::CopilotCredentialEnvelope>(
                login.credentials.clone().unwrap(),
            )
            .unwrap()
            .pending_login,
            scope: None,
            session: None,
        });

        let status = authed_client.auth_status(&profile).await.unwrap();
        assert_eq!(
            status.status.state,
            gunmetal_core::ProviderAuthState::Connected
        );

        let ready_client = CopilotClient::with_options(CopilotClientOptions {
            api_base_url: github.uri(),
            client_id: "client-id".to_owned(),
            copilot_base_url: copilot.uri(),
            login_base_url: github.uri(),
            pending_login: None,
            scope: None,
            session: serde_json::from_value::<super::CopilotCredentialEnvelope>(
                status.credentials.clone().unwrap(),
            )
            .unwrap()
            .session,
        });

        let models = ready_client.list_models(&profile).await.unwrap();
        assert_eq!(models.models[0].id, "copilot/gpt-5.4");

        let completion = ready_client
            .chat_completion(
                &profile,
                &gunmetal_core::ChatCompletionRequest {
                    model: "copilot/gpt-5.4".to_owned(),
                    messages: vec![gunmetal_core::ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(completion.completion.message.content, "GUNMETAL_COPILOT_OK");
        assert_eq!(completion.completion.usage.total_tokens, Some(5));
    }
}
