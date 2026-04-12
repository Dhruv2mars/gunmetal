use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures_util::{
    StreamExt,
    stream::{self, BoxStream},
};
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ModelDescriptor, ModelMetadata,
    ProviderAuthStatus, ProviderKind, ProviderLoginSession, ProviderProfile, TokenUsage,
};
use gunmetal_storage::AppPaths;
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderStreamEvent {
    TextDelta(String),
    Complete {
        model: String,
        finish_reason: String,
        usage: TokenUsage,
    },
}

pub type ProviderEventStream = BoxStream<'static, Result<ProviderStreamEvent>>;
pub type ProviderByteStream = BoxStream<'static, Result<Vec<u8>>>;

pub struct ProviderStreamResult {
    pub stream: ProviderEventStream,
    pub credentials: Option<Value>,
}

pub struct ProviderRawSseResult {
    pub stream: ProviderByteStream,
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

    async fn stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderStreamResult> {
        let result = self.chat_completion(profile, paths, request).await?;
        Ok(ProviderStreamResult {
            credentials: result.credentials,
            stream: synthetic_completion_stream(result.completion),
        })
    }

    async fn raw_stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        paths: &AppPaths,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderRawSseResult> {
        let result = self.stream_chat_completion(profile, paths, request).await?;
        Ok(ProviderRawSseResult {
            credentials: result.credentials,
            stream: synthetic_chat_sse_stream(request.model.clone(), result.stream),
        })
    }
}

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    adapters: HashMap<ProviderKind, Arc<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
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
    pub fn new(paths: AppPaths, registry: ProviderRegistry) -> Self {
        Self {
            paths,
            registry,
            models_dev: ModelsDevCatalog::default(),
        }
    }

    pub fn with_registry(paths: AppPaths, registry: ProviderRegistry) -> Self {
        Self::new(paths, registry)
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

    pub async fn stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderEventStream> {
        let adapter = self.adapter(&profile.provider)?;
        let result = adapter
            .stream_chat_completion(profile, &self.paths, request)
            .await?;
        self.persist_credentials(profile.id, result.credentials)?;
        Ok(result.stream)
    }

    pub async fn raw_stream_chat_completion(
        &self,
        profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ProviderByteStream> {
        let adapter = self.adapter(&profile.provider)?;
        let result = adapter
            .raw_stream_chat_completion(profile, &self.paths, request)
            .await?;
        self.persist_credentials(profile.id, result.credentials)?;
        Ok(result.stream)
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
        let Some(credentials) = credentials else {
            return Ok(());
        };
        self.paths
            .storage_handle()?
            .update_profile_credentials(profile_id, Some(credentials))
    }
}

fn synthetic_completion_stream(completion: ChatCompletionResult) -> ProviderEventStream {
    let mut events = text_chunks(&completion.message.content)
        .into_iter()
        .map(ProviderStreamEvent::TextDelta)
        .collect::<Vec<_>>();
    events.push(ProviderStreamEvent::Complete {
        model: completion.model,
        finish_reason: completion.finish_reason,
        usage: completion.usage,
    });
    stream::iter(events.into_iter().map(Ok)).boxed()
}

pub fn synthetic_chat_sse_stream(model: String, stream: ProviderEventStream) -> ProviderByteStream {
    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4().simple());
    let created = chrono::Utc::now().timestamp();
    let first = stream::once(async move {
        Ok::<Vec<u8>, anyhow::Error>(
            format!(
                "data: {}\n\n",
                serde_json::json!({
                    "id": id,
                    "object": "chat.completion.chunk",
                    "created": created,
                    "model": model,
                    "choices": [{
                        "index": 0,
                        "delta": { "role": "assistant" },
                        "finish_reason": Value::Null
                    }]
                })
            )
            .into_bytes(),
        )
    });

    let content = stream.map(move |item| match item {
        Ok(ProviderStreamEvent::TextDelta(chunk)) => Ok(format!(
            "data: {}\n\n",
            serde_json::json!({
                "id": format!("chatcmpl-{}", uuid::Uuid::new_v4().simple()),
                "object": "chat.completion.chunk",
                "created": chrono::Utc::now().timestamp(),
                "choices": [{
                    "index": 0,
                    "delta": { "content": chunk },
                    "finish_reason": Value::Null
                }]
            })
        )
        .into_bytes()),
        Ok(ProviderStreamEvent::Complete {
            model,
            finish_reason,
            usage,
        }) => Ok(format!(
            "data: {}\n\n",
            serde_json::json!({
                "id": format!("chatcmpl-{}", uuid::Uuid::new_v4().simple()),
                "object": "chat.completion.chunk",
                "created": chrono::Utc::now().timestamp(),
                "model": model,
                "choices": [{
                    "index": 0,
                    "delta": {},
                    "finish_reason": finish_reason
                }],
                "usage": usage
            })
        )
        .into_bytes()),
        Err(error) => Ok(format!(
            "event: error\ndata: {}\n\n",
            serde_json::json!({ "error": { "message": error.to_string() } })
        )
        .into_bytes()),
    });

    let done = stream::once(async { Ok::<Vec<u8>, anyhow::Error>(b"data: [DONE]\n\n".to_vec()) });
    first.chain(content).chain(done).boxed()
}

pub fn openai_compatible_event_stream<F>(
    response: Response,
    fallback_model: String,
    normalize_model: F,
) -> ProviderEventStream
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    let normalize_model = Arc::new(normalize_model);
    async_stream::try_stream! {
        let mut upstream = response.bytes_stream();
        let mut decoder = SseDecoder::default();
        let mut current_model = fallback_model;

        while let Some(chunk) = upstream.next().await {
            let chunk = chunk?;
            decoder.push(&chunk);

            while let Some(event) = decoder.next_event() {
                if event == "[DONE]" {
                    continue;
                }

                for parsed in parse_openai_compatible_event(
                    &event,
                    &mut current_model,
                    normalize_model.as_ref(),
                )? {
                    yield parsed;
                }
            }
        }
    }
    .boxed()
}

fn parse_openai_compatible_event(
    event: &str,
    current_model: &mut String,
    normalize_model: &dyn Fn(&str) -> String,
) -> Result<Vec<ProviderStreamEvent>> {
    let payload = serde_json::from_str::<OpenAiCompatibleStreamChunk>(event)?;
    if let Some(model) = payload.model.as_deref() {
        *current_model = normalize_model(model);
    }

    let mut events = Vec::new();
    let usage = payload.usage.map(to_token_usage);
    for choice in payload.choices {
        if let Some(delta) = choice.delta.and_then(|delta| delta.content)
            && !delta.is_empty()
        {
            events.push(ProviderStreamEvent::TextDelta(delta));
        }

        if let Some(finish_reason) = choice.finish_reason {
            events.push(ProviderStreamEvent::Complete {
                model: current_model.clone(),
                finish_reason,
                usage: usage.clone().unwrap_or(TokenUsage {
                    input_tokens: None,
                    output_tokens: None,
                    total_tokens: None,
                }),
            });
        }
    }

    Ok(events)
}

fn to_token_usage(usage: OpenAiCompatibleUsage) -> TokenUsage {
    let input_tokens = usage.prompt_tokens.map(to_u32);
    let output_tokens = usage.completion_tokens.map(to_u32);
    let total_tokens =
        usage
            .total_tokens
            .map(to_u32)
            .or_else(|| match (input_tokens, output_tokens) {
                (Some(input), Some(output)) => Some(input.saturating_add(output)),
                _ => None,
            });

    TokenUsage {
        input_tokens,
        output_tokens,
        total_tokens,
    }
}

#[derive(Default)]
struct SseDecoder {
    buffer: String,
}

impl SseDecoder {
    fn push(&mut self, chunk: &[u8]) {
        let chunk = String::from_utf8_lossy(chunk);
        let chunk = chunk.replace("\r\n", "\n");
        self.buffer.push_str(&chunk);
    }

    fn next_event(&mut self) -> Option<String> {
        let separator = self.buffer.find("\n\n")?;
        let frame = self.buffer[..separator].to_owned();
        self.buffer.drain(..separator + 2);

        let data = frame
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n");
        (!data.is_empty()).then_some(data)
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleStreamChunk {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<OpenAiCompatibleStreamChoice>,
    #[serde(default)]
    usage: Option<OpenAiCompatibleUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleStreamChoice {
    #[serde(default)]
    delta: Option<OpenAiCompatibleStreamDelta>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleStreamDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiCompatibleUsage {
    #[serde(default)]
    prompt_tokens: Option<u64>,
    #[serde(default)]
    completion_tokens: Option<u64>,
    #[serde(default)]
    total_tokens: Option<u64>,
}

fn text_chunks(value: &str) -> Vec<String> {
    if value.is_empty() {
        return vec![String::new()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut count = 0usize;
    for ch in value.chars() {
        current.push(ch);
        count += 1;
        if count >= 24 {
            chunks.push(std::mem::take(&mut current));
            count = 0;
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
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
            http: Client::builder()
                .connect_timeout(std::time::Duration::from_secs(2))
                .timeout(std::time::Duration::from_secs(4))
                .build()
                .expect("reqwest client"),
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

#[cfg(test)]
mod tests {
    use anyhow::{Result, bail};
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
        let hub = ProviderHub::new(paths.clone(), registry);

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
