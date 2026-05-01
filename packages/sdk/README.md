# gunmetal-sdk

Provider adapter SDK for adding upstream providers to Gunmetal.

Install:

```bash
cargo add gunmetal-sdk gunmetal-core gunmetal-storage
```

## What This Crate Provides

- `ProviderAdapter`: async trait for provider integrations
- `ProviderRegistry`: registry of adapters by `ProviderKind`
- `ProviderHub`: high-level auth, model sync, chat, streaming, and credential-persistence orchestration
- `ProviderDefinition`: public provider metadata and capabilities
- streaming helpers for OpenAI-compatible server-sent events
- model enrichment helpers through `ModelsDevCatalog`

## Minimal Adapter Shape

```rust
use anyhow::Result;
use async_trait::async_trait;
use gunmetal_core::{
    ChatCompletionRequest, ProviderAuthStatus, ProviderKind, ProviderLoginSession,
    ProviderProfile,
};
use gunmetal_sdk::{
    ProviderAdapter, ProviderAuthMethod, ProviderAuthResult, ProviderCapabilities,
    ProviderChatResult, ProviderClass, ProviderDefinition, ProviderLoginResult,
    ProviderModelSyncResult, ProviderUxHints,
};
use gunmetal_storage::AppPaths;

pub struct MyProvider;

#[async_trait]
impl ProviderAdapter for MyProvider {
    fn definition(&self) -> ProviderDefinition {
        ProviderDefinition {
            kind: ProviderKind::Custom("my-provider".to_owned()),
            label: "my-provider",
            class: ProviderClass::Gateway,
            priority: 100,
            capabilities: ProviderCapabilities {
                auth_method: ProviderAuthMethod::ApiKey,
                supports_base_url: true,
                supports_model_sync: true,
                supports_chat_completions: true,
                supports_responses_api: false,
                supports_streaming: true,
            },
            ux: ProviderUxHints {
                helper_title: "API-key provider",
                helper_body: "Paste an API key and sync models.",
                suggested_name: "my-provider",
                base_url_placeholder: "https://api.example.com/v1",
            },
        }
    }

    async fn auth_status(
        &self,
        _profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderAuthResult> {
        Ok(ProviderAuthResult {
            credentials: None,
            status: ProviderAuthStatus::Connected,
        })
    }

    async fn login(
        &self,
        _profile: &ProviderProfile,
        _paths: &AppPaths,
        _open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        Ok(ProviderLoginResult {
            credentials: None,
            session: ProviderLoginSession {
                auth_url: String::new(),
                user_code: None,
                expires_at: None,
            },
        })
    }

    async fn logout(
        &self,
        _profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }

    async fn sync_models(
        &self,
        _profile: &ProviderProfile,
        _paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult> {
        Ok(ProviderModelSyncResult {
            credentials: None,
            models: Vec::new(),
        })
    }

    async fn chat_completion(
        &self,
        _profile: &ProviderProfile,
        _paths: &AppPaths,
        _request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        anyhow::bail!("chat completion not configured for this example")
    }
}
```

## Registering an Adapter

```rust
use gunmetal_sdk::{ProviderHub, ProviderRegistry};
use gunmetal_storage::AppPaths;

let paths = AppPaths::resolve()?;
let mut registry = ProviderRegistry::default();
registry.register(MyProvider);

let hub = ProviderHub::new(paths, registry);
```

Applications can use `ProviderHub` to call auth status, login, model sync, chat completions, and streaming completions while letting Gunmetal persist refreshed credentials back into local state.
