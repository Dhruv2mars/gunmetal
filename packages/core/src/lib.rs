use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ProviderKind {
    Codex,
    Copilot,
    OpenRouter,
    Zen,
    OpenAi,
    Azure,
    Nvidia,
    Custom(String),
}

impl ProviderKind {
    pub fn slug(&self) -> Cow<'_, str> {
        match self {
            Self::Codex => Cow::Borrowed("codex"),
            Self::Copilot => Cow::Borrowed("copilot"),
            Self::OpenRouter => Cow::Borrowed("openrouter"),
            Self::Zen => Cow::Borrowed("zen"),
            Self::OpenAi => Cow::Borrowed("openai"),
            Self::Azure => Cow::Borrowed("azure"),
            Self::Nvidia => Cow::Borrowed("nvidia"),
            Self::Custom(value) => Cow::Borrowed(value.as_str()),
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.slug())
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "codex" => Ok(Self::Codex),
            "copilot" => Ok(Self::Copilot),
            "openrouter" => Ok(Self::OpenRouter),
            "zen" => Ok(Self::Zen),
            "openai" => Ok(Self::OpenAi),
            "azure" => Ok(Self::Azure),
            "nvidia" => Ok(Self::Nvidia),
            value if !value.trim().is_empty() => Ok(Self::Custom(value.to_owned())),
            _ => Err("provider kind cannot be empty".to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyScope {
    Inference,
    ModelsRead,
    LogsRead,
}

impl std::fmt::Display for KeyScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Inference => "inference",
            Self::ModelsRead => "models_read",
            Self::LogsRead => "logs_read",
        };

        write!(f, "{value}")
    }
}

impl std::str::FromStr for KeyScope {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "inference" => Ok(Self::Inference),
            "models_read" => Ok(Self::ModelsRead),
            "logs_read" => Ok(Self::LogsRead),
            _ => Err(format!("unknown key scope: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyState {
    Active,
    Disabled,
    Revoked,
}

impl std::fmt::Display for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        };

        write!(f, "{value}")
    }
}

impl std::str::FromStr for KeyState {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            "revoked" => Ok(Self::Revoked),
            _ => Err(format!("unknown key state: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GunmetalKey {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub state: KeyState,
    pub scopes: Vec<KeyScope>,
    pub allowed_providers: Vec<ProviderKind>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl GunmetalKey {
    pub fn can_access_provider(&self, provider: &ProviderKind) -> bool {
        self.allowed_providers.is_empty()
            || self.allowed_providers.iter().any(|item| item == provider)
    }

    pub fn is_usable_at(&self, now: DateTime<Utc>) -> bool {
        self.state == KeyState::Active && self.expires_at.is_none_or(|value| value > now)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewGunmetalKey {
    pub name: String,
    pub scopes: Vec<KeyScope>,
    pub allowed_providers: Vec<ProviderKind>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedGunmetalKey {
    pub record: GunmetalKey,
    pub secret: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub id: Uuid,
    pub provider: ProviderKind,
    pub name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub credentials: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewProviderProfile {
    pub provider: ProviderKind,
    pub name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub credentials: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub provider: ProviderKind,
    pub profile_id: Option<Uuid>,
    pub upstream_name: String,
    pub display_name: String,
    pub metadata: Option<ModelMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelMetadata {
    pub family: Option<String>,
    pub release_date: Option<String>,
    pub last_updated: Option<String>,
    #[serde(default)]
    pub input_modalities: Vec<String>,
    #[serde(default)]
    pub output_modalities: Vec<String>,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub supports_attachments: Option<bool>,
    pub supports_reasoning: Option<bool>,
    pub supports_tools: Option<bool>,
    pub open_weights: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for ChatRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        };
        write!(f, "{value}")
    }
}

impl std::str::FromStr for ChatRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            _ => Err(format!("unknown chat role: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequestMode {
    #[default]
    Normalized,
    Passthrough,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RequestOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_output_tokens: Option<u32>,
    #[serde(default)]
    pub stop: Vec<String>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(default)]
    pub provider_options: Map<String, Value>,
    #[serde(default)]
    pub mode: RequestMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(default)]
    pub options: RequestOptions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionResult {
    pub model: String,
    pub message: ChatMessage,
    pub finish_reason: String,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAuthState {
    SignedOut,
    SigningIn,
    Connected,
    Expired,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAuthStatus {
    pub state: ProviderAuthState,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderLoginSession {
    pub login_id: String,
    pub auth_url: String,
    pub user_code: Option<String>,
    pub interval_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestLogEntry {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub key_id: Option<Uuid>,
    pub profile_id: Option<Uuid>,
    pub provider: ProviderKind,
    pub model: String,
    pub endpoint: String,
    pub status_code: Option<u16>,
    pub duration_ms: u64,
    pub usage: TokenUsage,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewRequestLogEntry {
    pub key_id: Option<Uuid>,
    pub profile_id: Option<Uuid>,
    pub provider: ProviderKind,
    pub model: String,
    pub endpoint: String,
    pub status_code: Option<u16>,
    pub duration_ms: u64,
    pub usage: TokenUsage,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn provider_parses_known_and_custom_variants() {
        assert_eq!(
            "codex".parse::<ProviderKind>().unwrap(),
            ProviderKind::Codex
        );
        assert_eq!(
            "edgebox".parse::<ProviderKind>().unwrap(),
            ProviderKind::Custom("edgebox".to_owned())
        );
    }

    #[test]
    fn active_key_checks_state_expiry_and_provider() {
        let now = Utc::now();
        let key = GunmetalKey {
            id: Uuid::new_v4(),
            name: "default".to_owned(),
            prefix: "gm_test".to_owned(),
            state: KeyState::Active,
            scopes: vec![KeyScope::Inference],
            allowed_providers: vec![ProviderKind::Codex],
            expires_at: Some(now + Duration::hours(1)),
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        assert!(key.can_access_provider(&ProviderKind::Codex));
        assert!(!key.can_access_provider(&ProviderKind::Copilot));
        assert!(key.is_usable_at(now));
        assert!(!key.is_usable_at(now + Duration::hours(2)));
    }

    #[test]
    fn chat_role_parses_known_values() {
        assert_eq!("user".parse::<ChatRole>().unwrap(), ChatRole::User);
        assert!("tool".parse::<ChatRole>().is_err());
    }

    #[test]
    fn request_options_default_to_normalized_mode() {
        let options = RequestOptions::default();
        assert_eq!(options.mode, RequestMode::Normalized);
        assert!(options.provider_options.is_empty());
        assert!(options.metadata.is_empty());
    }
}
