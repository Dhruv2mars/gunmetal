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

    #[test]
    fn gunmetal_key_roundtrip() {
        let now = Utc::now();
        let original = GunmetalKey {
            id: Uuid::new_v4(),
            name: "test-key".to_owned(),
            prefix: "gm_test".to_owned(),
            state: KeyState::Active,
            scopes: vec![KeyScope::Inference, KeyScope::ModelsRead],
            allowed_providers: vec![ProviderKind::Codex, ProviderKind::Custom("edge".to_owned())],
            expires_at: Some(now + Duration::hours(1)),
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: GunmetalKey = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn provider_profile_roundtrip() {
        let now = Utc::now();
        let original = ProviderProfile {
            id: Uuid::new_v4(),
            provider: ProviderKind::OpenAi,
            name: "openai".to_owned(),
            base_url: Some("https://api.openai.com".to_owned()),
            enabled: true,
            credentials: Some(serde_json::json!({"key": "secret"})),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ProviderProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn model_descriptor_roundtrip() {
        let original = ModelDescriptor {
            id: "openai/gpt-4".to_owned(),
            provider: ProviderKind::OpenAi,
            profile_id: Some(Uuid::new_v4()),
            upstream_name: "gpt-4".to_owned(),
            display_name: "GPT-4".to_owned(),
            metadata: Some(ModelMetadata {
                family: Some("gpt".to_owned()),
                release_date: Some("2023-03-14".to_owned()),
                last_updated: None,
                input_modalities: vec!["text".to_owned()],
                output_modalities: vec!["text".to_owned()],
                context_window: Some(8192),
                max_output_tokens: Some(4096),
                supports_attachments: Some(false),
                supports_reasoning: Some(true),
                supports_tools: Some(true),
                open_weights: Some(false),
            }),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ModelDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn token_usage_roundtrip() {
        let original = TokenUsage {
            input_tokens: Some(10),
            output_tokens: Some(20),
            total_tokens: Some(30),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TokenUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn request_options_roundtrip() {
        let mut metadata = Map::new();
        metadata.insert(
            "user".to_owned(),
            serde_json::Value::String("alice".to_owned()),
        );
        let original = RequestOptions {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_output_tokens: Some(256),
            stop: vec!["STOP".to_owned()],
            metadata,
            provider_options: Map::new(),
            mode: RequestMode::Passthrough,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RequestOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn chat_completion_request_roundtrip() {
        let original = ChatCompletionRequest {
            model: "gpt-4".to_owned(),
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: "You are helpful.".to_owned(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: "Hello".to_owned(),
                },
            ],
            stream: true,
            options: RequestOptions::default(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ChatCompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn chat_completion_result_roundtrip() {
        let original = ChatCompletionResult {
            model: "gpt-4".to_owned(),
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: "Hi there!".to_owned(),
            },
            finish_reason: "stop".to_owned(),
            usage: TokenUsage {
                input_tokens: Some(1),
                output_tokens: Some(2),
                total_tokens: Some(3),
            },
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ChatCompletionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn chat_message_roundtrip() {
        let original = ChatMessage {
            role: ChatRole::User,
            content: "test".to_owned(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn provider_auth_status_roundtrip() {
        let original = ProviderAuthStatus {
            state: ProviderAuthState::Connected,
            label: "Connected to OpenAI".to_owned(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ProviderAuthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn chat_message_empty_content_roundtrip() {
        let original = ChatMessage {
            role: ChatRole::Assistant,
            content: "".to_owned(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn token_usage_missing_fields_deserialize() {
        let json = r#"{"input_tokens":10}"#;
        let deserialized: TokenUsage = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.input_tokens, Some(10));
        assert_eq!(deserialized.output_tokens, None);
        assert_eq!(deserialized.total_tokens, None);
    }

    #[test]
    fn request_options_defaults_when_missing() {
        let json = r#"{"temperature":0.5}"#;
        let deserialized: RequestOptions = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.temperature, Some(0.5));
        assert!(deserialized.stop.is_empty());
        assert!(deserialized.metadata.is_empty());
        assert!(deserialized.provider_options.is_empty());
        assert_eq!(deserialized.mode, RequestMode::Normalized);
    }

    #[test]
    fn model_descriptor_null_metadata() {
        let json = r#"{
            "id": "openai/gpt-4",
            "provider": {"kind":"open_ai","value":null},
            "profile_id": null,
            "upstream_name": "gpt-4",
            "display_name": "GPT-4",
            "metadata": null
        }"#;
        let deserialized: ModelDescriptor = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.metadata, None);
    }

    #[test]
    fn provider_auth_state_enum_variants() {
        for state in [
            ProviderAuthState::SignedOut,
            ProviderAuthState::SigningIn,
            ProviderAuthState::Connected,
            ProviderAuthState::Expired,
            ProviderAuthState::Error,
        ] {
            let status = ProviderAuthStatus {
                state,
                label: "test".to_owned(),
            };
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: ProviderAuthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn provider_kind_enum_variants() {
        for kind in [
            ProviderKind::Codex,
            ProviderKind::Copilot,
            ProviderKind::OpenRouter,
            ProviderKind::Zen,
            ProviderKind::OpenAi,
            ProviderKind::Azure,
            ProviderKind::Nvidia,
            ProviderKind::Custom("x".to_owned()),
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            let deserialized: ProviderKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, deserialized);
        }
    }
}
