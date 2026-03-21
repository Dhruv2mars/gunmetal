use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ModelDescriptor, ModelMetadata,
    ProviderAuthStatus, ProviderKind, ProviderLoginSession, ProviderProfile,
};
use gunmetal_storage::AppPaths;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;

mod codex;
mod copilot;
mod openai;
mod openrouter;
mod zen;

pub use codex::{CodexClient, CodexClientOptions};
pub use copilot::{CopilotClient, CopilotClientOptions, CopilotSession};
pub use openai::{OpenAiClient, OpenAiClientOptions};
pub use openrouter::{OpenRouterClient, OpenRouterClientOptions};
pub use zen::{ZenClient, ZenClientOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderClass {
    Subscription,
    Gateway,
    Direct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDefinition {
    pub kind: ProviderKind,
    pub class: ProviderClass,
    pub priority: usize,
}

#[derive(Debug, Clone)]
pub struct ProviderAuthResult {
    pub credentials: Option<Value>,
    pub status: ProviderAuthStatus,
}

#[derive(Debug, Clone)]
pub struct ProviderLoginResult {
    pub credentials: Option<Value>,
    pub session: ProviderLoginSession,
}

#[derive(Debug, Clone)]
pub struct ProviderModelSyncResult {
    pub credentials: Option<Value>,
    pub models: Vec<ModelDescriptor>,
}

#[derive(Debug, Clone)]
pub struct ProviderChatResult {
    pub completion: ChatCompletionResult,
    pub credentials: Option<Value>,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn definition(&self) -> ProviderDefinition;

    async fn auth_status(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
    ) -> Result<ProviderAuthResult>;

    async fn login(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        open_browser: bool,
    ) -> Result<ProviderLoginResult>;

    async fn logout(&self, profile: &ProviderProfile, paths: &AppPaths) -> Result<Option<Value>>;

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult>;

    async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult>;
}

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    adapters: HashMap<ProviderKind, Arc<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::default();
        registry.register(CodexAdapter);
        registry.register(CopilotAdapter);
        registry.register(OpenRouterAdapter);
        registry.register(ZenAdapter);
        registry.register(OpenAiAdapter);
        registry
    }

    pub fn register<A>(&mut self, adapter: A)
    where
        A: ProviderAdapter + 'static,
    {
        let adapter = Arc::new(adapter);
        self.adapters
            .insert(adapter.definition().kind.clone(), adapter);
    }

    pub fn get(&self, kind: &ProviderKind) -> Option<Arc<dyn ProviderAdapter>> {
        self.adapters.get(kind).cloned()
    }

    pub fn definitions(&self) -> Vec<ProviderDefinition> {
        let mut definitions = self
            .adapters
            .values()
            .map(|adapter| adapter.definition())
            .collect::<Vec<_>>();
        definitions.sort_by_key(|item| item.priority);
        definitions
    }
}

#[derive(Clone)]
pub struct ProviderHub {
    paths: AppPaths,
    registry: ProviderRegistry,
    models_dev: ModelsDevCatalog,
}

impl ProviderHub {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            registry: ProviderRegistry::with_defaults(),
            models_dev: ModelsDevCatalog::default(),
        }
    }

    pub fn with_registry(paths: AppPaths, registry: ProviderRegistry) -> Self {
        Self {
            paths,
            registry,
            models_dev: ModelsDevCatalog::default(),
        }
    }

    pub fn with_registry_and_models_dev(
        paths: AppPaths,
        registry: ProviderRegistry,
        models_dev: ModelsDevCatalog,
    ) -> Self {
        Self {
            paths,
            registry,
            models_dev,
        }
    }

    pub async fn auth_status(&self, profile: &ProviderProfile) -> Result<ProviderAuthStatus> {
        let adapter = self.adapter(&profile.provider)?;
        let result = adapter.auth_status(profile, &self.paths).await?;
        self.persist_credentials(profile.id, result.credentials)?;
        Ok(result.status)
    }

    pub async fn login(
        &self,
        profile: &ProviderProfile,
        open_browser: bool,
    ) -> Result<ProviderLoginSession> {
        let adapter = self.adapter(&profile.provider)?;
        let result = adapter.login(profile, &self.paths, open_browser).await?;
        self.persist_credentials(profile.id, result.credentials)?;
        Ok(result.session)
    }

    pub async fn logout(&self, profile: &ProviderProfile) -> Result<()> {
        let adapter = self.adapter(&profile.provider)?;
        let credentials = adapter.logout(profile, &self.paths).await?;
        self.persist_credentials(profile.id, credentials)
    }

    pub async fn sync_models(&self, profile: &ProviderProfile) -> Result<Vec<ModelDescriptor>> {
        let adapter = self.adapter(&profile.provider)?;
        let mut result = adapter.sync_models(profile, &self.paths).await?;
        self.persist_credentials(profile.id, result.credentials)?;
        if let Err(error) = self.models_dev.enrich(&mut result.models).await {
            let _ = error;
        }
        Ok(result.models)
    }

    pub async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResult> {
        let adapter = self.adapter(&profile.provider)?;
        let result = adapter
            .chat_completion(profile, &self.paths, request)
            .await?;
        self.persist_credentials(profile.id, result.credentials)?;
        Ok(result.completion)
    }

    fn adapter(&self, kind: &ProviderKind) -> Result<Arc<dyn ProviderAdapter>> {
        self.registry
            .get(kind)
            .ok_or_else(|| anyhow!("provider '{}' not implemented yet", kind))
    }

    fn persist_credentials(
        &self,
        profile_id: uuid::Uuid,
        credentials: Option<serde_json::Value>,
    ) -> Result<()> {
        self.paths
            .storage_handle()?
            .update_profile_credentials(profile_id, credentials)
    }
}

pub fn builtin_providers() -> Vec<ProviderDefinition> {
    ProviderRegistry::with_defaults().definitions()
}

#[derive(Clone)]
pub struct ModelsDevCatalog {
    catalog_url: String,
    http: Client,
    cache: Arc<Mutex<Option<ModelsDevIndex>>>,
}

impl Default for ModelsDevCatalog {
    fn default() -> Self {
        Self::new("https://models.dev/api.json")
    }
}

impl ModelsDevCatalog {
    pub fn new(catalog_url: impl Into<String>) -> Self {
        Self {
            catalog_url: catalog_url.into(),
            http: Client::builder().build().expect("reqwest client"),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    async fn enrich(&self, models: &mut [ModelDescriptor]) -> Result<()> {
        let index = self.index().await?;
        for model in models {
            if model.metadata.is_some() {
                continue;
            }

            let aliases = provider_aliases(&model.provider);
            let metadata = aliases
                .iter()
                .find_map(|alias| index.by_provider.get(*alias))
                .and_then(|models| models.get(&model.upstream_name).cloned())
                .or_else(|| index.by_model_id.get(&model.upstream_name).cloned());
            model.metadata = metadata;
        }
        Ok(())
    }

    async fn index(&self) -> Result<ModelsDevIndex> {
        {
            let cache = self.cache.lock().await;
            if let Some(index) = cache.as_ref() {
                return Ok(index.clone());
            }
        }

        let payload = self
            .http
            .get(&self.catalog_url)
            .send()
            .await?
            .error_for_status()?
            .json::<HashMap<String, ModelsDevProvider>>()
            .await?;
        let index = ModelsDevIndex::from_payload(payload);
        let mut cache = self.cache.lock().await;
        *cache = Some(index.clone());
        Ok(index)
    }
}

#[derive(Debug, Clone, Default)]
struct ModelsDevIndex {
    by_model_id: HashMap<String, ModelMetadata>,
    by_provider: HashMap<String, HashMap<String, ModelMetadata>>,
}

impl ModelsDevIndex {
    fn from_payload(payload: HashMap<String, ModelsDevProvider>) -> Self {
        let mut index = Self::default();
        for (provider, envelope) in payload {
            let mut provider_models = HashMap::new();
            for (model_id, model) in envelope.models {
                let metadata = ModelMetadata {
                    family: model.family,
                    release_date: model.release_date,
                    last_updated: model.last_updated,
                    input_modalities: model.modalities.input,
                    output_modalities: model.modalities.output,
                    context_window: model.limit.context.map(to_u32),
                    max_output_tokens: model.limit.output.map(to_u32),
                    supports_attachments: model.attachment,
                    supports_reasoning: model.reasoning,
                    supports_tools: model.tool_call,
                    open_weights: model.open_weights,
                };
                provider_models.insert(model_id.clone(), metadata.clone());
                index.by_model_id.entry(model_id).or_insert(metadata);
            }
            index.by_provider.insert(provider, provider_models);
        }
        index
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModelsDevProvider {
    #[serde(default)]
    models: HashMap<String, ModelsDevModel>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModelsDevModel {
    family: Option<String>,
    attachment: Option<bool>,
    reasoning: Option<bool>,
    tool_call: Option<bool>,
    open_weights: Option<bool>,
    release_date: Option<String>,
    last_updated: Option<String>,
    #[serde(default)]
    modalities: ModelsDevModalities,
    #[serde(default)]
    limit: ModelsDevLimits,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModelsDevModalities {
    #[serde(default)]
    input: Vec<String>,
    #[serde(default)]
    output: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModelsDevLimits {
    context: Option<u64>,
    output: Option<u64>,
}

fn provider_aliases(provider: &ProviderKind) -> &'static [&'static str] {
    match provider {
        ProviderKind::Codex => &["codex", "openai"],
        ProviderKind::Copilot => &["copilot", "github"],
        ProviderKind::OpenRouter => &["openrouter"],
        ProviderKind::Zen => &["zen", "opencode", "zenmux"],
        ProviderKind::OpenAi => &["openai"],
        ProviderKind::Azure => &["azure", "azure-cognitive-services"],
        ProviderKind::Nvidia => &["nvidia"],
        ProviderKind::Custom(_) => &[],
    }
}

fn to_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

struct CodexAdapter;
struct CopilotAdapter;
struct OpenRouterAdapter;
struct ZenAdapter;
struct OpenAiAdapter;

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
        Ok(ProviderAuthResult {
            credentials: None,
            status: CodexClient::spawn(CodexClientOptions::from_profile(profile, paths))
                .await?
                .auth_status()
                .await?,
        })
    }

    async fn login(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        open_browser: bool,
    ) -> Result<ProviderLoginResult> {
        let session = CodexClient::spawn(CodexClientOptions::from_profile(profile, paths))
            .await?
            .login()
            .await?;
        if open_browser {
            let _ = webbrowser::open(&session.auth_url);
        }
        Ok(ProviderLoginResult {
            credentials: None,
            session,
        })
    }

    async fn logout(&self, profile: &ProviderProfile, paths: &AppPaths) -> Result<Option<Value>> {
        CodexClient::spawn(CodexClientOptions::from_profile(profile, paths))
            .await?
            .logout()
            .await?;
        Ok(None)
    }

    async fn sync_models(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
    ) -> Result<ProviderModelSyncResult> {
        Ok(ProviderModelSyncResult {
            credentials: None,
            models: CodexClient::spawn(CodexClientOptions::from_profile(profile, paths))
                .await?
                .list_models(profile.id)
                .await?,
        })
    }

    async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderChatResult> {
        Ok(ProviderChatResult {
            credentials: None,
            completion: CodexClient::spawn(CodexClientOptions::from_profile(profile, paths))
                .await?
                .chat_completion(profile.id, request)
                .await?,
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

    async fn logout(&self, profile: &ProviderProfile, _paths: &AppPaths) -> Result<Option<Value>> {
        let _ = profile;
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
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use gunmetal_core::{
        ChatMessage, ChatRole, NewProviderProfile, ProviderAuthState, RequestOptions,
    };
    use serde_json::json;
    use tempfile::TempDir;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

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

    #[tokio::test]
    async fn provider_hub_uses_registered_adapter_and_persists_credentials() {
        let temp = TempDir::new().unwrap();
        let paths = AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let storage = paths.storage_handle().unwrap();
        let profile = storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::Custom("mock".to_owned()),
                name: "mock".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();

        let mut registry = ProviderRegistry::default();
        registry.register(MockAdapter);
        let hub = ProviderHub::with_registry(paths.clone(), registry);

        let status = hub.auth_status(&profile).await.unwrap();
        assert_eq!(status.state, ProviderAuthState::Connected);

        let synced = hub.sync_models(&profile).await.unwrap();
        assert_eq!(synced[0].id, "mock/model-1");

        let completion = hub
            .chat_completion(
                &profile,
                &ChatCompletionRequest {
                    model: "mock/model-1".to_owned(),
                    messages: vec![ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: false,
                    options: RequestOptions::default(),
                },
            )
            .await
            .unwrap();
        assert_eq!(completion.message.content, "hello from mock");

        let updated = storage.get_profile(profile.id).unwrap().unwrap();
        assert_eq!(updated.credentials, Some(json!({ "token": "updated" })));
    }

    #[tokio::test]
    async fn models_dev_enriches_synced_models() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "openai": {
                    "models": {
                        "gpt-5.1": {
                            "family": "gpt",
                            "attachment": true,
                            "reasoning": true,
                            "tool_call": true,
                            "open_weights": false,
                            "release_date": "2025-01-01",
                            "last_updated": "2025-02-01",
                            "modalities": { "input": ["text"], "output": ["text"] },
                            "limit": { "context": 272000, "output": 16384 }
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let temp = TempDir::new().unwrap();
        let paths = AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let storage = paths.storage_handle().unwrap();
        let profile = storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::Codex,
                name: "codex".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();

        let mut registry = ProviderRegistry::default();
        registry.register(MockCodexAdapter);
        let hub = ProviderHub::with_registry_and_models_dev(
            paths,
            registry,
            ModelsDevCatalog::new(format!("{}/api.json", server.uri())),
        );

        let models = hub.sync_models(&profile).await.unwrap();
        assert_eq!(
            models[0]
                .metadata
                .as_ref()
                .and_then(|value| value.family.as_deref()),
            Some("gpt")
        );
        assert_eq!(
            models[0]
                .metadata
                .as_ref()
                .and_then(|value| value.context_window),
            Some(272_000)
        );
    }

    #[derive(Default)]
    struct MockAdapter;

    #[async_trait]
    impl ProviderAdapter for MockAdapter {
        fn definition(&self) -> ProviderDefinition {
            ProviderDefinition {
                kind: ProviderKind::Custom("mock".to_owned()),
                class: ProviderClass::Direct,
                priority: 99,
            }
        }

        async fn auth_status(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
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
            _paths: &AppPaths,
            _open_browser: bool,
        ) -> Result<ProviderLoginResult> {
            bail!("not implemented")
        }

        async fn logout(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
        ) -> Result<Option<Value>> {
            Ok(None)
        }

        async fn sync_models(
            &self,
            profile: &ProviderProfile,
            _paths: &AppPaths,
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
            _paths: &AppPaths,
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
                    usage: gunmetal_core::TokenUsage {
                        input_tokens: Some(1),
                        output_tokens: Some(1),
                        total_tokens: Some(2),
                    },
                },
            })
        }
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
            _profile: &ProviderProfile,
            _paths: &AppPaths,
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
            _paths: &AppPaths,
            _open_browser: bool,
        ) -> Result<ProviderLoginResult> {
            bail!("not implemented")
        }

        async fn logout(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
        ) -> Result<Option<Value>> {
            Ok(None)
        }

        async fn sync_models(
            &self,
            profile: &ProviderProfile,
            _paths: &AppPaths,
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
            _paths: &AppPaths,
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
                    usage: gunmetal_core::TokenUsage {
                        input_tokens: Some(1),
                        output_tokens: Some(1),
                        total_tokens: Some(2),
                    },
                },
            })
        }
    }
}
