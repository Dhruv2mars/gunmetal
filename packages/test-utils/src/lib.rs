use anyhow::Result;
use async_trait::async_trait;
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ChatMessage, ChatRole, ModelDescriptor,
    ProviderAuthState, ProviderAuthStatus, ProviderContext, ProviderKind, ProviderLoginSession,
    ProviderProfile, TokenUsage,
};
use gunmetal_sdk::{
    ProviderAdapter, ProviderAuthMethod, ProviderAuthResult, ProviderCapabilities,
    ProviderChatResult, ProviderClass, ProviderDefinition, ProviderEventStream,
    ProviderLoginResult, ProviderModelSyncResult, ProviderRawSseResult, ProviderStreamResult,
    ProviderUxHints,
};
use serde_json::{Value, json};

pub fn provider_definition_fixture(
    kind: ProviderKind,
    class: ProviderClass,
    priority: usize,
) -> ProviderDefinition {
    let (label, auth_method, supports_base_url, helper_title, helper_body, base_url_placeholder) =
        match kind {
            ProviderKind::Codex => (
                "codex",
                ProviderAuthMethod::BrowserSession,
                false,
                "Browser sign-in provider",
                "Save the provider, then auth it in the browser.",
                "not used for this provider",
            ),
            ProviderKind::Copilot => (
                "copilot",
                ProviderAuthMethod::BrowserSession,
                false,
                "Browser sign-in provider",
                "Save the provider, then auth it in the browser.",
                "not used for this provider",
            ),
            ProviderKind::OpenRouter => (
                "openrouter",
                ProviderAuthMethod::ApiKey,
                true,
                "Gateway provider",
                "Save the upstream API key here.",
                "https://openrouter.ai/api/v1",
            ),
            ProviderKind::Zen => (
                "zen",
                ProviderAuthMethod::ApiKey,
                true,
                "Gateway provider",
                "Save the upstream API key here.",
                "https://opencode.ai/zen/v1",
            ),
            ProviderKind::OpenAi => (
                "openai",
                ProviderAuthMethod::ApiKey,
                true,
                "Direct provider",
                "Save the upstream API key here.",
                "https://api.openai.com/v1",
            ),
            ProviderKind::Custom(_) | ProviderKind::Azure | ProviderKind::Nvidia => (
                "custom",
                ProviderAuthMethod::ApiKey,
                true,
                "Direct provider",
                "Save the upstream API key here.",
                "optional override",
            ),
        };
    ProviderDefinition {
        kind,
        label,
        class,
        priority,
        capabilities: ProviderCapabilities {
            auth_method,
            supports_base_url,
            supports_model_sync: true,
            supports_chat_completions: true,
            supports_responses_api: true,
            supports_streaming: true,
        },
        ux: ProviderUxHints {
            helper_title,
            helper_body,
            suggested_name: label,
            base_url_placeholder,
        },
    }
}

#[derive(Default)]
pub struct MockAdapter;

#[async_trait]
impl ProviderAdapter for MockAdapter {
    fn definition(&self) -> ProviderDefinition {
        provider_definition_fixture(
            ProviderKind::Custom("mock".to_owned()),
            ProviderClass::Direct,
            99,
        )
    }

    async fn auth_status(
        &self,
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
    ) -> Result<ProviderAuthResult> {
        Ok(ProviderAuthResult {
            credentials: Some(json!({ "token": "updated" })),
            status: ProviderAuthStatus {
                state: ProviderAuthState::Connected,
                label: "mock".to_owned(),
            },
        })
    }

    async fn login(
        &self,
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
        _open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        anyhow::bail!("not implemented")
    }

    async fn logout(
        &self,
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
    ) -> Result<Option<Value>> {
        Ok(None)
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
    ) -> Result<ProviderModelSyncResult> {
        Ok(ProviderModelSyncResult {
            credentials: Some(json!({ "token": "updated" })),
            models: vec![ModelDescriptor {
                id: "mock/model-1".to_owned(),
                provider: profile.provider.clone(),
                profile_id: Some(profile.id),
                upstream_name: "model-1".to_owned(),
                display_name: "Model 1".to_owned(),
                metadata: None,
            }],
        })
    }

    async fn chat_completion(
        &self,
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        Ok(ProviderChatResult {
            credentials: Some(json!({ "token": "updated" })),
            completion: ChatCompletionResult {
                model: request.model.clone(),
                message: ChatMessage {
                    role: ChatRole::Assistant,
                    content: "hello from mock".to_owned(),
                },
                finish_reason: "stop".to_owned(),
                usage: TokenUsage {
                    input_tokens: Some(1),
                    output_tokens: Some(1),
                    total_tokens: Some(2),
                },
            },
        })
    }
}

#[derive(Default)]
pub struct MockCodexAdapter;

#[async_trait]
impl ProviderAdapter for MockCodexAdapter {
    fn definition(&self) -> ProviderDefinition {
        provider_definition_fixture(ProviderKind::Codex, ProviderClass::Subscription, 1)
    }

    async fn auth_status(
        &self,
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
    ) -> Result<ProviderAuthResult> {
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
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
        _open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        anyhow::bail!("not implemented")
    }

    async fn logout(
        &self,
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
    ) -> Result<Option<Value>> {
        Ok(None)
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
    ) -> Result<ProviderModelSyncResult> {
        Ok(ProviderModelSyncResult {
            credentials: None,
            models: vec![ModelDescriptor {
                id: "codex/gpt-5.1".to_owned(),
                provider: ProviderKind::Codex,
                profile_id: Some(profile.id),
                upstream_name: "gpt-5.1".to_owned(),
                display_name: "GPT-5.1".to_owned(),
                metadata: None,
            }],
        })
    }

    async fn chat_completion(
        &self,
        _profile: &ProviderProfile,
        _paths: &dyn ProviderContext,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        Ok(ProviderChatResult {
            credentials: None,
            completion: ChatCompletionResult {
                model: request.model.clone(),
                message: ChatMessage {
                    role: ChatRole::Assistant,
                    content: "hello".to_owned(),
                },
                finish_reason: "stop".to_owned(),
                usage: TokenUsage {
                    input_tokens: Some(1),
                    output_tokens: Some(1),
                    total_tokens: Some(2),
                },
            },
        })
    }
}
