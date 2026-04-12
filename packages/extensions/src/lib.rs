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
    ModelsDevCatalog, ProviderAdapter, ProviderAuthResult, ProviderByteStream, ProviderChatResult,
    ProviderClass, ProviderDefinition, ProviderEventStream, ProviderHub, ProviderLoginResult,
    ProviderModelSyncResult, ProviderRawSseResult, ProviderRegistry, ProviderStreamEvent,
    ProviderStreamResult, openai_compatible_event_stream, synthetic_chat_sse_stream,
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
            class: ProviderClass::Subscription,
            priority: 1,
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
            class: ProviderClass::Subscription,
            priority: 2,
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
            class: ProviderClass::Gateway,
            priority: 3,
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
            class: ProviderClass::Gateway,
            priority: 4,
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
            class: ProviderClass::Direct,
            priority: 5,
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
    }
}
