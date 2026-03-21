use std::{future::Future, pin::Pin, sync::Arc};

use anyhow::{Result, bail};
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ModelDescriptor, ProviderAuthStatus, ProviderKind,
    ProviderLoginSession, ProviderProfile,
};
use gunmetal_storage::AppPaths;

mod codex;
mod copilot;

pub use codex::{CodexClient, CodexClientOptions};
pub use copilot::{CopilotClient, CopilotClientOptions, CopilotSession};

type CodexConnector = Arc<
    dyn Fn(ProviderProfile, AppPaths) -> Pin<Box<dyn Future<Output = Result<CodexClient>> + Send>>
        + Send
        + Sync,
>;
type CopilotConnector = Arc<
    dyn Fn(ProviderProfile, AppPaths) -> Pin<Box<dyn Future<Output = Result<CopilotClient>> + Send>>
        + Send
        + Sync,
>;

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

#[derive(Clone)]
pub struct ProviderHub {
    paths: AppPaths,
    codex_connector: CodexConnector,
    copilot_connector: CopilotConnector,
}

impl ProviderHub {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            codex_connector: Arc::new(|profile, paths| {
                Box::pin(async move {
                    CodexClient::spawn(CodexClientOptions::from_profile(&profile, &paths)).await
                })
            }),
            copilot_connector: Arc::new(|profile, _paths| {
                Box::pin(async move {
                    Ok(CopilotClient::with_options(
                        CopilotClientOptions::from_profile(&profile),
                    ))
                })
            }),
        }
    }

    pub fn with_connectors(
        paths: AppPaths,
        codex_connector: CodexConnector,
        copilot_connector: CopilotConnector,
    ) -> Self {
        Self {
            paths,
            codex_connector,
            copilot_connector,
        }
    }

    pub fn with_codex_connector(paths: AppPaths, codex_connector: CodexConnector) -> Self {
        let copilot_connector = Arc::new(
            move |_profile: ProviderProfile,
                  _paths: AppPaths|
                  -> Pin<Box<dyn Future<Output = Result<CopilotClient>> + Send>> {
                Box::pin(async move { Ok(CopilotClient::mock("hello from copilot")) })
            },
        );
        Self::with_connectors(paths, codex_connector, copilot_connector)
    }

    pub fn with_copilot_connector(paths: AppPaths, copilot_connector: CopilotConnector) -> Self {
        let codex_connector = Arc::new(
            move |_profile: ProviderProfile,
                  _paths: AppPaths|
                  -> Pin<Box<dyn Future<Output = Result<CodexClient>> + Send>> {
                Box::pin(async move { Ok(CodexClient::mock("hello from codex")) })
            },
        );
        Self::with_connectors(paths, codex_connector, copilot_connector)
    }

    pub async fn auth_status(&self, profile: &ProviderProfile) -> Result<ProviderAuthStatus> {
        match profile.provider {
            ProviderKind::Codex => self.codex(profile).await?.auth_status().await,
            ProviderKind::Copilot => {
                let result = self.copilot(profile).await?.auth_status(profile).await?;
                self.persist_credentials(profile.id, result.credentials)?;
                Ok(result.status)
            }
            _ => bail!(
                "provider '{}' auth status not implemented yet",
                profile.provider
            ),
        }
    }

    pub async fn login(
        &self,
        profile: &ProviderProfile,
        open_browser: bool,
    ) -> Result<ProviderLoginSession> {
        match profile.provider {
            ProviderKind::Codex => {
                let session = self.codex(profile).await?.login().await?;
                if open_browser {
                    let _ = webbrowser::open(&session.auth_url);
                }
                Ok(session)
            }
            ProviderKind::Copilot => {
                let result = self
                    .copilot(profile)
                    .await?
                    .login(profile, open_browser)
                    .await?;
                self.persist_credentials(profile.id, result.credentials)?;
                Ok(result.session)
            }
            _ => bail!("provider '{}' login not implemented yet", profile.provider),
        }
    }

    pub async fn logout(&self, profile: &ProviderProfile) -> Result<()> {
        match profile.provider {
            ProviderKind::Codex => self.codex(profile).await?.logout().await,
            ProviderKind::Copilot => {
                self.persist_credentials(profile.id, None)?;
                Ok(())
            }
            _ => bail!("provider '{}' logout not implemented yet", profile.provider),
        }
    }

    pub async fn sync_models(&self, profile: &ProviderProfile) -> Result<Vec<ModelDescriptor>> {
        match profile.provider {
            ProviderKind::Codex => self.codex(profile).await?.list_models(profile.id).await,
            ProviderKind::Copilot => {
                let result = self.copilot(profile).await?.list_models(profile).await?;
                self.persist_credentials(profile.id, result.credentials)?;
                Ok(result.models)
            }
            _ => bail!(
                "provider '{}' model sync not implemented yet",
                profile.provider
            ),
        }
    }

    pub async fn chat_completion(
        &self,
        profile: &ProviderProfile,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResult> {
        match profile.provider {
            ProviderKind::Codex => {
                self.codex(profile)
                    .await?
                    .chat_completion(profile.id, request)
                    .await
            }
            ProviderKind::Copilot => {
                let result = self
                    .copilot(profile)
                    .await?
                    .chat_completion(profile, request)
                    .await?;
                self.persist_credentials(profile.id, result.credentials)?;
                Ok(result.completion)
            }
            _ => bail!(
                "provider '{}' chat completions not implemented yet",
                profile.provider
            ),
        }
    }

    async fn codex(&self, profile: &ProviderProfile) -> Result<CodexClient> {
        (self.codex_connector)(profile.clone(), self.paths.clone()).await
    }

    async fn copilot(&self, profile: &ProviderProfile) -> Result<CopilotClient> {
        (self.copilot_connector)(profile.clone(), self.paths.clone()).await
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
    vec![
        ProviderDefinition {
            kind: ProviderKind::Codex,
            class: ProviderClass::Subscription,
            priority: 1,
        },
        ProviderDefinition {
            kind: ProviderKind::Copilot,
            class: ProviderClass::Subscription,
            priority: 2,
        },
        ProviderDefinition {
            kind: ProviderKind::OpenRouter,
            class: ProviderClass::Gateway,
            priority: 3,
        },
        ProviderDefinition {
            kind: ProviderKind::Zen,
            class: ProviderClass::Gateway,
            priority: 4,
        },
        ProviderDefinition {
            kind: ProviderKind::OpenAi,
            class: ProviderClass::Direct,
            priority: 5,
        },
        ProviderDefinition {
            kind: ProviderKind::Azure,
            class: ProviderClass::Direct,
            priority: 6,
        },
        ProviderDefinition {
            kind: ProviderKind::Nvidia,
            class: ProviderClass::Direct,
            priority: 7,
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::{future::Future, pin::Pin, sync::Arc};

    use anyhow::Result;
    use chrono::Utc;
    use gunmetal_core::{
        ChatCompletionRequest, ChatCompletionResult, ChatMessage, ChatRole, ProviderAuthState,
        ProviderKind, ProviderProfile, TokenUsage,
    };
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::{CodexClient, CopilotClient, ProviderClass, ProviderHub, builtin_providers};

    #[test]
    fn builtin_provider_order_matches_product_priority() {
        let providers = builtin_providers();
        assert_eq!(providers[0].kind, ProviderKind::Codex);
        assert_eq!(providers[1].kind, ProviderKind::Copilot);
        assert_eq!(providers[2].kind, ProviderKind::OpenRouter);
        assert_eq!(providers[3].kind, ProviderKind::Zen);
    }

    #[test]
    fn provider_classes_are_partitioned() {
        let providers = builtin_providers();
        assert_eq!(providers[0].class, ProviderClass::Subscription);
        assert_eq!(providers[2].class, ProviderClass::Gateway);
        assert_eq!(providers[4].class, ProviderClass::Direct);
    }

    #[tokio::test]
    async fn provider_hub_delegates_to_codex_connector() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let codex_connector = Arc::new(
            move |_profile: ProviderProfile,
                  _paths: gunmetal_storage::AppPaths|
                  -> Pin<Box<dyn Future<Output = Result<CodexClient>> + Send>> {
                Box::pin(async move { Ok(CodexClient::mock("hello from codex")) })
            },
        );
        let copilot_connector = Arc::new(
            move |_profile: ProviderProfile,
                  _paths: gunmetal_storage::AppPaths|
                  -> Pin<Box<dyn Future<Output = Result<CopilotClient>> + Send>> {
                Box::pin(async move { Ok(CopilotClient::mock("hello from copilot")) })
            },
        );
        let hub = ProviderHub::with_connectors(paths, codex_connector, copilot_connector);
        let profile = ProviderProfile {
            id: Uuid::new_v4(),
            provider: ProviderKind::Codex,
            name: "default".to_owned(),
            base_url: None,
            enabled: true,
            credentials: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let status = hub.auth_status(&profile).await.unwrap();
        assert_eq!(status.state, ProviderAuthState::Connected);

        let models = hub.sync_models(&profile).await.unwrap();
        assert_eq!(models[0].id, "codex/gpt-5.4");

        let response = hub
            .chat_completion(
                &profile,
                &ChatCompletionRequest {
                    model: "codex/gpt-5.4".to_owned(),
                    messages: vec![ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(response.message.content, "hello from codex");
    }

    #[tokio::test]
    async fn provider_hub_delegates_to_copilot_connector() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let codex_connector = Arc::new(
            move |_profile: ProviderProfile,
                  _paths: gunmetal_storage::AppPaths|
                  -> Pin<Box<dyn Future<Output = Result<CodexClient>> + Send>> {
                Box::pin(async move { Ok(CodexClient::mock("hello from codex")) })
            },
        );
        let copilot_connector = Arc::new(
            move |_profile: ProviderProfile,
                  _paths: gunmetal_storage::AppPaths|
                  -> Pin<Box<dyn Future<Output = Result<CopilotClient>> + Send>> {
                Box::pin(async move { Ok(CopilotClient::mock("hello from copilot")) })
            },
        );
        let hub = ProviderHub::with_connectors(paths.clone(), codex_connector, copilot_connector);
        let storage = paths.storage_handle().unwrap();
        let profile = storage
            .create_profile(gunmetal_core::NewProviderProfile {
                provider: ProviderKind::Copilot,
                name: "default".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();

        let status = hub.auth_status(&profile).await.unwrap();
        assert_eq!(status.state, ProviderAuthState::Connected);

        let models = hub.sync_models(&profile).await.unwrap();
        assert_eq!(models[0].id, "copilot/gpt-5.4");

        let response = hub
            .chat_completion(
                &profile,
                &ChatCompletionRequest {
                    model: "copilot/gpt-5.4".to_owned(),
                    messages: vec![ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(response.message.content, "hello from copilot");
    }

    #[tokio::test]
    async fn mock_shapes_are_sane() {
        let codex = CodexClient::mock("done");
        let copilot = CopilotClient::mock("done");
        let _ = ChatCompletionResult {
            model: "codex/gpt-5.4".to_owned(),
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: "done".to_owned(),
            },
            finish_reason: "stop".to_owned(),
            usage: TokenUsage {
                input_tokens: Some(1),
                output_tokens: Some(1),
                total_tokens: Some(2),
            },
        };
        assert!(codex.is_mock());
        assert!(copilot.is_mock());
    }
}
