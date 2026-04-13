use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, bail};
use async_trait::async_trait;
use gunmetal_core::{ChatCompletionRequest, ProviderKind, ProviderProfile};
use gunmetal_storage::AppPaths;
use serde_json::Value;
use tokio::sync::Mutex;

mod codex;
mod copilot;
mod openai;
mod openrouter;
mod zen;

pub use codex::{CodexClient, CodexClientOptions};
pub use copilot::{CopilotClient, CopilotClientOptions, CopilotSession};
pub use gunmetal_sdk::{
    ModelsDevCatalog, ProviderAdapter, ProviderAuthMethod, ProviderAuthResult, ProviderByteStream,
    ProviderCapabilities, ProviderChatResult, ProviderClass, ProviderDefinition,
    ProviderEventStream, ProviderHub, ProviderLoginResult, ProviderModelSyncResult,
    ProviderRawSseResult, ProviderRegistry, ProviderStreamEvent, ProviderStreamResult,
    ProviderUxHints, openai_compatible_event_stream, synthetic_chat_sse_stream,
};
pub use openai::{OpenAiClient, OpenAiClientOptions};
pub use openrouter::{OpenRouterClient, OpenRouterClientOptions};
pub use zen::{ZenClient, ZenClientOptions};

#[derive(Clone, Default)]
struct CodexAdapter {
    clients: Arc<Mutex<HashMap<uuid::Uuid, Arc<Mutex<CodexClient>>>>>,
}

struct CopilotAdapter;
struct OpenRouterAdapter;
struct ZenAdapter;
struct OpenAiAdapter;

impl CodexAdapter {
    async fn cached_client(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
    ) -> Result<Arc<Mutex<CodexClient>>> {
        {
            let clients = self.clients.lock().await;
            if let Some(client) = clients.get(&profile.id) {
                return Ok(client.clone());
            }
        }

        let client = Arc::new(Mutex::new(
            CodexClient::spawn(CodexClientOptions::from_profile(profile, paths)).await?,
        ));
        let mut clients = self.clients.lock().await;
        Ok(clients
            .entry(profile.id)
            .or_insert_with(|| client.clone())
            .clone())
    }

    async fn evict_client(&self, profile_id: uuid::Uuid) {
        self.clients.lock().await.remove(&profile_id);
    }
}

pub fn builtin_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::default();
    registry.register(CodexAdapter::default());
    registry.register(CopilotAdapter);
    registry.register(OpenRouterAdapter);
    registry.register(ZenAdapter);
    registry.register(OpenAiAdapter);
    registry
}

pub fn builtin_provider_hub(paths: AppPaths) -> ProviderHub {
    ProviderHub::new(paths, builtin_registry())
}

pub fn builtin_providers() -> Vec<ProviderDefinition> {
    builtin_registry().definitions()
}

#[async_trait]
impl ProviderAdapter for CodexAdapter {
    fn definition(&self) -> ProviderDefinition {
        ProviderDefinition {
            kind: ProviderKind::Codex,
            label: "codex",
            class: ProviderClass::Subscription,
            priority: 1,
            capabilities: ProviderCapabilities {
                auth_method: ProviderAuthMethod::BrowserSession,
                supports_base_url: false,
                supports_model_sync: true,
                supports_chat_completions: true,
                supports_responses_api: true,
                supports_streaming: true,
            },
            ux: ProviderUxHints {
                helper_title: "Browser sign-in provider",
                helper_body: "Save the provider, then auth it through the browser flow. Base URL and API key are not needed here.",
                suggested_name: "codex",
                base_url_placeholder: "not used for this provider",
            },
        }
    }

    async fn auth_status(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
    ) -> Result<ProviderAuthResult> {
        let client = self.cached_client(profile, paths).await?;
        let client = client.lock().await;
        Ok(ProviderAuthResult {
            credentials: None,
            status: client.auth_status().await?,
        })
    }

    async fn login(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        let client = self.cached_client(profile, paths).await?;
        let client = client.lock().await;
        let session = client.login().await?;
        if open_browser {
            let _ = webbrowser::open(&session.auth_url);
        }
        Ok(ProviderLoginResult {
            credentials: None,
            session,
        })
    }

    async fn logout(&self, profile: &ProviderProfile, paths: &AppPaths) -> Result<Option<Value>> {
        let client = self.cached_client(profile, paths).await?;
        let client = client.lock().await;
        client.logout().await?;
        drop(client);
        self.evict_client(profile.id).await;
        Ok(None)
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult> {
        let client = self.cached_client(profile, paths).await?;
        let client = client.lock().await;
        Ok(ProviderModelSyncResult {
            credentials: None,
            models: client.list_models(profile.id).await?,
        })
    }

    async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        let client = self.cached_client(profile, paths).await?;
        let client = client.lock().await;
        Ok(ProviderChatResult {
            credentials: None,
            completion: client.chat_completion(profile.id, request).await?,
        })
    }
}

#[async_trait]
impl ProviderAdapter for CopilotAdapter {
    fn definition(&self) -> ProviderDefinition {
        ProviderDefinition {
            kind: ProviderKind::Copilot,
            label: "copilot",
            class: ProviderClass::Subscription,
            priority: 2,
            capabilities: ProviderCapabilities {
                auth_method: ProviderAuthMethod::BrowserSession,
                supports_base_url: false,
                supports_model_sync: true,
                supports_chat_completions: true,
                supports_responses_api: true,
                supports_streaming: true,
            },
            ux: ProviderUxHints {
                helper_title: "Browser sign-in provider",
                helper_body: "Save the provider, then auth it through the browser flow. Base URL and API key are not needed here.",
                suggested_name: "copilot",
                base_url_placeholder: "not used for this provider",
            },
        }
    }

    async fn auth_status(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderAuthResult> {
        let result = CopilotClient::with_options(CopilotClientOptions::from_profile(profile))
            .auth_status(profile)
            .await?;
        Ok(ProviderAuthResult {
            credentials: result.credentials,
            status: result.status,
        })
    }

    async fn login(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        let result = CopilotClient::with_options(CopilotClientOptions::from_profile(profile))
            .login(profile, open_browser)
            .await?;
        Ok(ProviderLoginResult {
            credentials: result.credentials,
            session: result.session,
        })
    }

    async fn logout(&self, _profile: &ProviderProfile, _paths: &AppPaths) -> Result<Option<Value>> {
        Ok(None)
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult> {
        let result = CopilotClient::with_options(CopilotClientOptions::from_profile(profile))
            .list_models(profile)
            .await?;
        Ok(ProviderModelSyncResult {
            credentials: result.credentials,
            models: result.models,
        })
    }

    async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        let result = CopilotClient::with_options(CopilotClientOptions::from_profile(profile))
            .chat_completion(profile, request)
            .await?;
        Ok(ProviderChatResult {
            credentials: result.credentials,
            completion: result.completion,
        })
    }
}

#[async_trait]
impl ProviderAdapter for OpenRouterAdapter {
    fn definition(&self) -> ProviderDefinition {
        ProviderDefinition {
            kind: ProviderKind::OpenRouter,
            label: "openrouter",
            class: ProviderClass::Gateway,
            priority: 3,
            capabilities: ProviderCapabilities {
                auth_method: ProviderAuthMethod::ApiKey,
                supports_base_url: true,
                supports_model_sync: true,
                supports_chat_completions: true,
                supports_responses_api: true,
                supports_streaming: true,
            },
            ux: ProviderUxHints {
                helper_title: "Gateway provider",
                helper_body: "Save your upstream API key here. Base URL usually stays on the default OpenRouter endpoint.",
                suggested_name: "openrouter",
                base_url_placeholder: "https://openrouter.ai/api/v1",
            },
        }
    }

    async fn auth_status(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderAuthResult> {
        let result = OpenRouterClient::with_options(OpenRouterClientOptions::from_profile(profile))
            .auth_status(profile)
            .await?;
        Ok(ProviderAuthResult {
            credentials: result.credentials,
            status: result.status,
        })
    }

    async fn login(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        _open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        bail!(
            "provider '{}' does not support browser login",
            profile.provider
        )
    }

    async fn logout(&self, profile: &ProviderProfile, _paths: &AppPaths) -> Result<Option<Value>> {
        Ok(
            OpenRouterClient::with_options(OpenRouterClientOptions::from_profile(profile))
                .clear_credentials(),
        )
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult> {
        let result = OpenRouterClient::with_options(OpenRouterClientOptions::from_profile(profile))
            .list_models(profile)
            .await?;
        Ok(ProviderModelSyncResult {
            credentials: result.credentials,
            models: result.models,
        })
    }

    async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        let result = OpenRouterClient::with_options(OpenRouterClientOptions::from_profile(profile))
            .chat_completion(profile, request)
            .await?;
        Ok(ProviderChatResult {
            credentials: result.credentials,
            completion: result.completion,
        })
    }

    async fn stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderStreamResult> {
        let result = OpenRouterClient::with_options(OpenRouterClientOptions::from_profile(profile))
            .stream_chat_completion(profile, request)
            .await?;
        Ok(ProviderStreamResult {
            credentials: result.credentials,
            stream: result.stream,
        })
    }

    async fn raw_stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderRawSseResult> {
        let client = OpenRouterClient::with_options(OpenRouterClientOptions::from_profile(profile));
        Ok(ProviderRawSseResult {
            credentials: None,
            stream: client.raw_stream_chat_completion(profile, request).await?,
        })
    }
}

#[async_trait]
impl ProviderAdapter for ZenAdapter {
    fn definition(&self) -> ProviderDefinition {
        ProviderDefinition {
            kind: ProviderKind::Zen,
            label: "zen",
            class: ProviderClass::Gateway,
            priority: 4,
            capabilities: ProviderCapabilities {
                auth_method: ProviderAuthMethod::ApiKey,
                supports_base_url: true,
                supports_model_sync: true,
                supports_chat_completions: true,
                supports_responses_api: true,
                supports_streaming: true,
            },
            ux: ProviderUxHints {
                helper_title: "Gateway provider",
                helper_body: "Save your upstream API key here. Base URL usually stays on the default Zen endpoint.",
                suggested_name: "zen",
                base_url_placeholder: "https://opencode.ai/zen/v1",
            },
        }
    }

    async fn auth_status(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderAuthResult> {
        let result = ZenClient::with_options(ZenClientOptions::from_profile(profile))
            .auth_status(profile)
            .await?;
        Ok(ProviderAuthResult {
            credentials: result.credentials,
            status: result.status,
        })
    }

    async fn login(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        _open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        bail!(
            "provider '{}' does not support browser login",
            profile.provider
        )
    }

    async fn logout(&self, profile: &ProviderProfile, _paths: &AppPaths) -> Result<Option<Value>> {
        Ok(ZenClient::with_options(ZenClientOptions::from_profile(profile)).clear_credentials())
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult> {
        let result = ZenClient::with_options(ZenClientOptions::from_profile(profile))
            .list_models(profile)
            .await?;
        Ok(ProviderModelSyncResult {
            credentials: result.credentials,
            models: result.models,
        })
    }

    async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        let result = ZenClient::with_options(ZenClientOptions::from_profile(profile))
            .chat_completion(profile, request)
            .await?;
        Ok(ProviderChatResult {
            credentials: result.credentials,
            completion: result.completion,
        })
    }
}

#[async_trait]
impl ProviderAdapter for OpenAiAdapter {
    fn definition(&self) -> ProviderDefinition {
        ProviderDefinition {
            kind: ProviderKind::OpenAi,
            label: "openai",
            class: ProviderClass::Direct,
            priority: 5,
            capabilities: ProviderCapabilities {
                auth_method: ProviderAuthMethod::ApiKey,
                supports_base_url: true,
                supports_model_sync: true,
                supports_chat_completions: true,
                supports_responses_api: true,
                supports_streaming: true,
            },
            ux: ProviderUxHints {
                helper_title: "Direct provider",
                helper_body: "Save your upstream API key here. Base URL is optional unless you need a custom endpoint.",
                suggested_name: "openai",
                base_url_placeholder: "https://api.openai.com/v1",
            },
        }
    }

    async fn auth_status(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderAuthResult> {
        Ok(ProviderAuthResult {
            credentials: profile.credentials.clone(),
            status: OpenAiClient::with_options(OpenAiClientOptions::from_profile(profile))
                .auth_status(profile)
                .await?,
        })
    }

    async fn login(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        _open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        bail!(
            "provider '{}' does not support browser login",
            profile.provider
        )
    }

    async fn logout(&self, profile: &ProviderProfile, _paths: &AppPaths) -> Result<Option<Value>> {
        Ok(
            OpenAiClient::with_options(OpenAiClientOptions::from_profile(profile))
                .clear_credentials(),
        )
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult> {
        Ok(ProviderModelSyncResult {
            credentials: profile.credentials.clone(),
            models: OpenAiClient::with_options(OpenAiClientOptions::from_profile(profile))
                .list_models(profile)
                .await?,
        })
    }

    async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        Ok(ProviderChatResult {
            credentials: profile.credentials.clone(),
            completion: OpenAiClient::with_options(OpenAiClientOptions::from_profile(profile))
                .chat_completion(profile, request)
                .await?,
        })
    }

    async fn stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderStreamResult> {
        Ok(ProviderStreamResult {
            credentials: profile.credentials.clone(),
            stream: OpenAiClient::with_options(OpenAiClientOptions::from_profile(profile))
                .stream_chat_completion(profile, request)
                .await?,
        })
    }

    async fn raw_stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        _paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderRawSseResult> {
        let client = OpenAiClient::with_options(OpenAiClientOptions::from_profile(profile));
        Ok(ProviderRawSseResult {
            credentials: profile.credentials.clone(),
            stream: client.raw_stream_chat_completion(profile, request).await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_provider_order_matches_product_priority() {
        let providers = builtin_providers();
        assert_eq!(providers[0].kind, ProviderKind::Codex);
        assert_eq!(providers[1].kind, ProviderKind::Copilot);
        assert_eq!(providers[2].kind, ProviderKind::OpenRouter);
        assert_eq!(providers[3].kind, ProviderKind::Zen);
        assert_eq!(providers[4].kind, ProviderKind::OpenAi);
        assert!(providers[0].supports_browser_login());
        assert!(!providers[0].capabilities.supports_base_url);
        assert!(providers[2].requires_api_key());
        assert_eq!(
            providers[2].ux.base_url_placeholder,
            "https://openrouter.ai/api/v1"
        );
    }
}
