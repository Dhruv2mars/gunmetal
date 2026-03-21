use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::{Context, Result, anyhow};
use gunmetal_core::{
    ChatCompletionRequest, ChatCompletionResult, ChatMessage, ChatRole, ModelDescriptor,
    ProviderAuthState, ProviderAuthStatus, ProviderLoginSession, ProviderProfile, TokenUsage,
};
use gunmetal_storage::AppPaths;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::{Mutex, oneshot},
};

const DEFAULT_DEVELOPER_INSTRUCTIONS: &str =
    "You are answering a normal conversational request. Do not perform file edits or tool actions.";

trait AsyncWriteTarget: AsyncWrite + Send + Unpin {}
impl<T> AsyncWriteTarget for T where T: AsyncWrite + Send + Unpin {}

type PendingResponseMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>>;

#[derive(Clone)]
pub struct CodexClient {
    _child: Arc<Mutex<Option<Child>>>,
    rpc: JsonRpcClient,
    mode: CodexMode,
}

#[derive(Clone)]
enum CodexMode {
    Live(CodexClientOptions),
    Mock(String),
}

#[derive(Debug, Clone)]
pub struct CodexClientOptions {
    pub client_name: String,
    pub client_title: String,
    pub client_version: String,
    pub codex_bin: PathBuf,
    pub cwd: PathBuf,
}

impl CodexClientOptions {
    pub fn from_profile(profile: &ProviderProfile, paths: &AppPaths) -> Self {
        let settings = profile
            .credentials
            .as_ref()
            .cloned()
            .and_then(|value| serde_json::from_value::<CodexProfileSettings>(value).ok())
            .unwrap_or_default();
        let fallback_helper = if cfg!(windows) {
            paths.helpers_dir.join("codex.exe")
        } else {
            paths.helpers_dir.join("codex")
        };
        let codex_bin = settings
            .bin_path
            .or_else(|| fallback_helper.exists().then_some(fallback_helper))
            .unwrap_or_else(|| PathBuf::from("codex"));

        Self {
            client_name: "gunmetal_daemon".to_owned(),
            client_title: "gunmetal".to_owned(),
            client_version: env!("CARGO_PKG_VERSION").to_owned(),
            codex_bin,
            cwd: settings
                .cwd
                .unwrap_or_else(|| paths.empty_workspace_dir.clone()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CodexProfileSettings {
    bin_path: Option<PathBuf>,
    cwd: Option<PathBuf>,
}

#[derive(Clone)]
struct JsonRpcClient {
    next_id: Arc<AtomicU64>,
    notifications: Arc<Mutex<VecDeque<RpcNotification>>>,
    pending: PendingResponseMap,
    writer: Arc<Mutex<Box<dyn AsyncWriteTarget>>>,
}

impl CodexClient {
    pub async fn spawn(options: CodexClientOptions) -> Result<Self> {
        let supports_listen = codex_supports_listen(&options.codex_bin).await;
        let mut command = Command::new(&options.codex_bin);
        command.arg("app-server");
        if supports_listen {
            command.arg("--listen").arg("stdio://");
        }
        let mut child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .with_context(|| format!("spawn {}", options.codex_bin.display()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("codex_stdin_unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("codex_stdout_unavailable"))?;

        let rpc = JsonRpcClient::new(stdout, stdin);
        let client = Self {
            _child: Arc::new(Mutex::new(Some(child))),
            rpc,
            mode: CodexMode::Live(options.clone()),
        };
        client
            .initialize(
                &options.client_name,
                &options.client_title,
                &options.client_version,
            )
            .await?;
        Ok(client)
    }

    pub fn from_parts<R, W>(reader: R, writer: W) -> Self
    where
        R: AsyncRead + Send + Unpin + 'static,
        W: AsyncWrite + Send + Unpin + 'static,
    {
        Self {
            _child: Arc::new(Mutex::new(None)),
            rpc: JsonRpcClient::new(reader, writer),
            mode: CodexMode::Live(CodexClientOptions {
                client_name: "gunmetal_test".to_owned(),
                client_title: "gunmetal".to_owned(),
                client_version: env!("CARGO_PKG_VERSION").to_owned(),
                codex_bin: PathBuf::from("codex"),
                cwd: PathBuf::from("."),
            }),
        }
    }

    pub fn mock(response: impl Into<String>) -> Self {
        let (reader, writer) = tokio::io::duplex(16);
        let (reader, _) = tokio::io::split(reader);
        let (_, writer) = tokio::io::split(writer);
        Self {
            _child: Arc::new(Mutex::new(None)),
            rpc: JsonRpcClient::new(reader, writer),
            mode: CodexMode::Mock(response.into()),
        }
    }

    pub fn is_mock(&self) -> bool {
        matches!(self.mode, CodexMode::Mock(_))
    }

    pub async fn auth_status(&self) -> Result<ProviderAuthStatus> {
        match &self.mode {
            CodexMode::Mock(_) => Ok(ProviderAuthStatus {
                state: ProviderAuthState::Connected,
                label: "mock@gunmetal (plus)".to_owned(),
            }),
            CodexMode::Live(_) => {
                let response: AccountReadResponse = self
                    .rpc
                    .request("account/read", json!({ "refreshToken": false }))
                    .await?;
                let state = if response.account.is_some() {
                    ProviderAuthState::Connected
                } else {
                    ProviderAuthState::SignedOut
                };
                let label = response
                    .account
                    .as_ref()
                    .map(|account| match (&account.email, &account.plan_type) {
                        (Some(email), Some(plan)) => format!("{email} ({plan})"),
                        (Some(email), None) => email.clone(),
                        _ => "ChatGPT".to_owned(),
                    })
                    .unwrap_or_else(|| "Signed out".to_owned());
                Ok(ProviderAuthStatus { state, label })
            }
        }
    }

    pub async fn login(&self) -> Result<ProviderLoginSession> {
        match &self.mode {
            CodexMode::Mock(_) => Ok(ProviderLoginSession {
                login_id: "mock-login".to_owned(),
                auth_url: "https://chatgpt.com".to_owned(),
                user_code: None,
                interval_seconds: None,
            }),
            CodexMode::Live(_) => {
                let response: LoginStartResponse = self
                    .rpc
                    .request("account/login/start", json!({ "type": "chatgpt" }))
                    .await?;
                Ok(ProviderLoginSession {
                    login_id: response.login_id,
                    auth_url: response.auth_url,
                    user_code: None,
                    interval_seconds: None,
                })
            }
        }
    }

    pub async fn logout(&self) -> Result<()> {
        if matches!(self.mode, CodexMode::Mock(_)) {
            return Ok(());
        }
        let _: Value = self.rpc.request("account/logout", json!({})).await?;
        Ok(())
    }

    pub async fn list_models(&self, profile_id: uuid::Uuid) -> Result<Vec<ModelDescriptor>> {
        match &self.mode {
            CodexMode::Mock(_) => Ok(vec![ModelDescriptor {
                id: "codex/gpt-5.4".to_owned(),
                provider: gunmetal_core::ProviderKind::Codex,
                profile_id: Some(profile_id),
                upstream_name: "gpt-5.4".to_owned(),
                display_name: "GPT-5.4".to_owned(),
                metadata: None,
            }]),
            CodexMode::Live(_) => {
                let response: ModelListResponse = self
                    .rpc
                    .request("model/list", json!({ "includeHidden": false }))
                    .await?;
                Ok(response
                    .data
                    .into_iter()
                    .filter(|item| !item.hidden.unwrap_or(false))
                    .map(|model| {
                        let upstream_name = model.id;
                        let display_name = model
                            .label
                            .or(model.display_name)
                            .unwrap_or_else(|| upstream_name.clone());
                        ModelDescriptor {
                            id: format!("codex/{upstream_name}"),
                            provider: gunmetal_core::ProviderKind::Codex,
                            profile_id: Some(profile_id),
                            upstream_name,
                            display_name,
                            metadata: None,
                        }
                    })
                    .collect())
            }
        }
    }

    pub async fn chat_completion(
        &self,
        _profile_id: uuid::Uuid,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResult> {
        match &self.mode {
            CodexMode::Mock(response) => Ok(ChatCompletionResult {
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
            }),
            CodexMode::Live(options) => {
                let model = request
                    .model
                    .strip_prefix("codex/")
                    .unwrap_or(&request.model)
                    .to_owned();
                let thread = self.thread_start(&model, &options.cwd).await?;
                let prompt = render_prompt(&request.messages);
                let result = self
                    .run_turn(&thread.id, &model, &options.cwd, &prompt)
                    .await?;

                Ok(ChatCompletionResult {
                    model: format!("codex/{model}"),
                    message: ChatMessage {
                        role: ChatRole::Assistant,
                        content: result.output,
                    },
                    finish_reason: "stop".to_owned(),
                    usage: result.usage,
                })
            }
        }
    }

    async fn initialize(&self, name: &str, title: &str, version: &str) -> Result<()> {
        if matches!(self.mode, CodexMode::Mock(_)) {
            return Ok(());
        }
        let _: Value = self
            .rpc
            .request(
                "initialize",
                json!({
                    "clientInfo": { "name": name, "title": title, "version": version }
                }),
            )
            .await?;
        self.rpc.notify("initialized", json!({})).await
    }

    async fn thread_start(&self, model: &str, cwd: &Path) -> Result<ThreadHandle> {
        let response: ThreadResponse = self
            .rpc
            .request(
                "thread/start",
                json!({
                    "approvalPolicy": "never",
                    "cwd": cwd,
                    "developerInstructions": DEFAULT_DEVELOPER_INSTRUCTIONS,
                    "model": model,
                    "personality": "friendly",
                    "sandbox": "read-only",
                    "serviceName": "gunmetal",
                }),
            )
            .await?;
        Ok(ThreadHandle {
            id: response.thread.id,
        })
    }

    async fn run_turn(
        &self,
        thread_id: &str,
        model: &str,
        cwd: &Path,
        prompt: &str,
    ) -> Result<TurnResult> {
        let response: TurnResponse = self
            .rpc
            .request(
                "turn/start",
                json!({
                    "approvalPolicy": "never",
                    "cwd": cwd,
                    "input": [{ "type": "text", "text": prompt }],
                    "model": model,
                    "personality": "friendly",
                    "sandboxPolicy": { "type": "readOnly", "networkAccess": false },
                    "summary": "concise",
                    "threadId": thread_id,
                }),
            )
            .await?;

        self.await_turn(thread_id, &response.turn.id).await
    }

    async fn await_turn(&self, thread_id: &str, turn_id: &str) -> Result<TurnResult> {
        let thread_id = thread_id.to_owned();
        let turn_id = turn_id.to_owned();
        let mut output = String::new();
        let mut usage = TokenUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
        };

        loop {
            let message = self.rpc.read_notification().await?;
            if !matches_thread_turn(&message.method, &message.params, &thread_id, &turn_id) {
                continue;
            }

            match message.method.as_str() {
                "item/agentMessage/delta" => {
                    if let Some(delta) = message.params.get("delta").and_then(Value::as_str) {
                        output.push_str(delta);
                    }
                }
                "thread/tokenUsage/updated" => {
                    if let Ok(payload) =
                        serde_json::from_value::<TokenUsageUpdatedNotification>(message.params)
                        && let Some(last) = payload.token_usage.last
                    {
                        usage = TokenUsage {
                            input_tokens: Some(last.input_tokens as u32),
                            output_tokens: Some(last.output_tokens as u32),
                            total_tokens: last.total_tokens.map(|value| value as u32),
                        };
                    }
                }
                "error" => {
                    let message = message
                        .params
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .unwrap_or("codex_turn_failed");
                    return Err(anyhow!(message.to_owned()));
                }
                "turn/completed" => return Ok(TurnResult { output, usage }),
                _ => {}
            }
        }
    }
}

async fn codex_supports_listen(bin: &Path) -> bool {
    let output = Command::new(bin)
        .arg("app-server")
        .arg("--help")
        .output()
        .await;
    let Ok(output) = output else {
        return true;
    };

    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.stderr.is_empty() {
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    text.contains("--listen")
}

fn render_prompt(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|message| format!("{}: {}", message.role, message.content.trim()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThreadHandle {
    id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TurnResult {
    output: String,
    usage: TokenUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RpcNotification {
    method: String,
    params: Value,
}

impl JsonRpcClient {
    fn new<R, W>(reader: R, writer: W) -> Self
    where
        R: AsyncRead + Send + Unpin + 'static,
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let notifications = Arc::new(Mutex::new(VecDeque::<RpcNotification>::new()));
        let pending_reader = pending.clone();
        let notifications_reader = notifications.clone();
        tokio::spawn(read_loop(
            BufReader::new(reader),
            pending_reader,
            notifications_reader,
        ));

        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            notifications,
            pending,
            writer: Arc::new(Mutex::new(Box::new(writer))),
        }
    }

    async fn notify(&self, method: &str, params: Value) -> Result<()> {
        self.write_message(&json!({ "method": method, "params": params }))
            .await
    }

    async fn request<T>(&self, method: &str, params: Value) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        self.write_message(&json!({ "id": id, "method": method, "params": params }))
            .await?;
        let response = rx
            .await
            .map_err(|_| anyhow!("rpc_response_dropped"))?
            .map_err(|error| anyhow!(error))?;
        Ok(serde_json::from_value(response)?)
    }

    async fn read_notification(&self) -> Result<RpcNotification> {
        loop {
            if let Some(notification) = self.notifications.lock().await.pop_front() {
                return Ok(notification);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    async fn write_message(&self, message: &Value) -> Result<()> {
        let mut writer = self.writer.lock().await;
        let mut bytes = serde_json::to_vec(message)?;
        bytes.push(b'\n');
        writer.write_all(&bytes).await?;
        writer.flush().await?;
        Ok(())
    }
}

async fn read_loop<R>(
    mut reader: BufReader<R>,
    pending: PendingResponseMap,
    notifications: Arc<Mutex<VecDeque<RpcNotification>>>,
) where
    R: AsyncRead + Send + Unpin + 'static,
{
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        if line.trim().is_empty() {
            continue;
        }

        let Ok(message) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        if let Some(method) = message.get("method").and_then(Value::as_str) {
            notifications.lock().await.push_back(RpcNotification {
                method: method.to_owned(),
                params: message.get("params").cloned().unwrap_or_else(|| json!({})),
            });
            continue;
        }

        let Some(id) = message.get("id").and_then(Value::as_u64) else {
            continue;
        };

        if let Some(sender) = pending.lock().await.remove(&id) {
            if let Some(error) = message.pointer("/error/message").and_then(Value::as_str) {
                let _ = sender.send(Err(error.to_owned()));
            } else {
                let _ = sender.send(Ok(message.get("result").cloned().unwrap_or(Value::Null)));
            }
        }
    }
}

fn matches_thread_turn(method: &str, params: &Value, thread_id: &str, turn_id: &str) -> bool {
    let matches_thread = params
        .get("threadId")
        .and_then(Value::as_str)
        .map(|value| value == thread_id)
        .unwrap_or_else(|| {
            params
                .pointer("/turn/threadId")
                .and_then(Value::as_str)
                .map(|value| value == thread_id)
                .unwrap_or(false)
        });
    if method == "thread/tokenUsage/updated" {
        return matches_thread;
    }
    matches_thread
        && params
            .get("turnId")
            .and_then(Value::as_str)
            .map(|value| value == turn_id)
            .unwrap_or_else(|| {
                params
                    .pointer("/turn/id")
                    .and_then(Value::as_str)
                    .map(|value| value == turn_id)
                    .unwrap_or(false)
            })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountReadResponse {
    account: Option<AccountPayload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountPayload {
    email: Option<String>,
    plan_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginStartResponse {
    auth_url: String,
    login_id: String,
}

#[derive(Debug, Deserialize)]
struct ThreadResponse {
    thread: RemoteThread,
}

#[derive(Debug, Deserialize)]
struct RemoteThread {
    id: String,
}

#[derive(Debug, Deserialize)]
struct TurnResponse {
    turn: RemoteTurn,
}

#[derive(Debug, Deserialize)]
struct RemoteTurn {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ModelListResponse {
    data: Vec<RemoteModel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteModel {
    display_name: Option<String>,
    hidden: Option<bool>,
    id: String,
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenUsageUpdatedNotification {
    token_usage: TokenUsageEnvelope,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenUsageEnvelope {
    last: Option<TokenUsageCounts>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenUsageCounts {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::{Value, json};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, duplex};

    use super::{CodexClient, render_prompt};
    use gunmetal_core::{ChatCompletionRequest, ChatMessage, ChatRole, RequestOptions};

    #[test]
    fn prompt_renderer_keeps_role_order() {
        let prompt = render_prompt(&[
            ChatMessage {
                role: ChatRole::System,
                content: "be concise".to_owned(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "ping".to_owned(),
            },
        ]);

        assert!(prompt.contains("system: be concise"));
        assert!(prompt.contains("user: ping"));
    }

    #[tokio::test]
    async fn codex_client_reads_models_and_completion_from_rpc() -> Result<()> {
        let (client_io, server_io) = duplex(8192);
        let (client_reader, client_writer) = tokio::io::split(client_io);
        let (server_reader, mut server_writer) = tokio::io::split(server_io);

        tokio::spawn(async move {
            let mut reader = BufReader::new(server_reader);
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).await.ok() == Some(0) {
                    break;
                }
                let request: Value = serde_json::from_str(&line).unwrap();
                let id = request.get("id").and_then(Value::as_u64);
                let method = request.get("method").and_then(Value::as_str).unwrap_or("");
                match method {
                    "initialize" => {
                        server_writer
                            .write_all(
                                format!("{}\n", json!({ "id": id, "result": {} })).as_bytes(),
                            )
                            .await
                            .unwrap();
                    }
                    "initialized" => {}
                    "account/read" => {
                        server_writer
                            .write_all(
                                format!(
                                    "{}\n",
                                    json!({
                                        "id": id,
                                        "result": { "account": { "email": "u@example.com", "planType": "plus" } }
                                    })
                                )
                                .as_bytes(),
                            )
                            .await
                            .unwrap();
                    }
                    "model/list" => {
                        server_writer
                            .write_all(
                                format!(
                                    "{}\n",
                                    json!({
                                        "id": id,
                                        "result": { "data": [{ "id": "gpt-5.4", "displayName": "GPT-5.4", "hidden": false }] }
                                    })
                                )
                                .as_bytes(),
                            )
                            .await
                            .unwrap();
                    }
                    "thread/start" => {
                        server_writer
                            .write_all(
                                format!(
                                    "{}\n",
                                    json!({ "id": id, "result": { "thread": { "id": "thr_1" } } })
                                )
                                .as_bytes(),
                            )
                            .await
                            .unwrap();
                    }
                    "turn/start" => {
                        server_writer
                            .write_all(
                                format!(
                                    "{}\n",
                                    json!({ "id": id, "result": { "turn": { "id": "turn_1" } } })
                                )
                                .as_bytes(),
                            )
                            .await
                            .unwrap();
                        for notification in [
                            json!({ "method": "item/agentMessage/delta", "params": { "threadId": "thr_1", "turnId": "turn_1", "delta": "Hello" } }),
                            json!({ "method": "thread/tokenUsage/updated", "params": { "threadId": "thr_1", "tokenUsage": { "last": { "inputTokens": 12, "outputTokens": 8, "totalTokens": 20 } } } }),
                            json!({ "method": "turn/completed", "params": { "threadId": "thr_1", "turnId": "turn_1" } }),
                        ] {
                            server_writer
                                .write_all(format!("{notification}\n").as_bytes())
                                .await
                                .unwrap();
                        }
                    }
                    _ => {}
                }
            }
        });

        let client = CodexClient::from_parts(client_reader, client_writer);
        client.initialize("gunmetal", "gunmetal", "0.1.0").await?;
        let status = client.auth_status().await?;
        assert_eq!(status.label, "u@example.com (plus)");

        let models = client.list_models(uuid::Uuid::nil()).await?;
        assert_eq!(models[0].id, "codex/gpt-5.4");

        let completion = client
            .chat_completion(
                uuid::Uuid::nil(),
                &ChatCompletionRequest {
                    model: "codex/gpt-5.4".to_owned(),
                    messages: vec![ChatMessage {
                        role: ChatRole::User,
                        content: "ping".to_owned(),
                    }],
                    stream: false,
                    options: RequestOptions::default(),
                },
            )
            .await?;
        assert_eq!(completion.message.content, "Hello");
        assert_eq!(completion.usage.total_tokens, Some(20));
        Ok(())
    }
}
