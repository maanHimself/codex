//! Runtime seams for embedding Codex in durable hosts.

use async_trait::async_trait;
use codex_async_utils::OrCancelExt;
use codex_exec_server::EnvironmentManager;
use codex_exec_server::ExecServerRuntimePaths;
use codex_extension_api::ExtensionRegistryBuilder;
use codex_extension_api::ThreadIdleInput;
use codex_extension_api::ThreadLifecycleContributor;
use codex_login::AuthManager;
use codex_otel::SessionTelemetry;
use codex_protocol::ThreadId;
use codex_protocol::config_types::ReasoningSummary as ReasoningSummaryConfig;
use codex_protocol::dynamic_tools::DynamicToolResponse;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
use codex_protocol::protocol::AgentStatus;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::ThreadGoal;
use codex_protocol::protocol::ThreadGoalStatus;
use codex_protocol::user_input::UserInput;
use codex_rollout_trace::InferenceTraceContext;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::client::ModelClient;
use crate::client::ModelClientSession;
use crate::client_common::Prompt;
use crate::client_common::ResponseStream;
use crate::codex_thread::CodexThread;
use crate::config::Config;
use crate::goals::SetGoalRequest;
use crate::installation_id::resolve_installation_id;
use crate::thread_manager::NewThread;
use crate::thread_manager::ThreadManager;
use crate::thread_store_from_config;

pub use codex_runtime_seams::DefaultIdGenerator;
pub use codex_runtime_seams::DefaultToolExecutionRuntime;
pub use codex_runtime_seams::EventSink;
pub use codex_runtime_seams::IdGenerator;
pub use codex_runtime_seams::NoopEventSink;
pub use codex_runtime_seams::ToolExecutionRequest;
pub use codex_runtime_seams::ToolExecutionRuntime;

pub struct ModelStreamRequest<'a> {
    pub prompt: &'a Prompt,
    pub model_info: &'a ModelInfo,
    pub session_telemetry: &'a SessionTelemetry,
    pub effort: Option<ReasoningEffortConfig>,
    pub summary: ReasoningSummaryConfig,
    pub service_tier: Option<String>,
    pub turn_metadata_header: Option<&'a str>,
    pub inference_trace: &'a InferenceTraceContext,
    pub cancellation_token: CancellationToken,
}

/// Turn-scoped model runtime. A fresh instance must be used per Codex turn.
#[async_trait]
pub trait ModelTurnRuntime: Send {
    async fn stream(&mut self, request: ModelStreamRequest<'_>) -> CodexResult<ResponseStream>;

    fn try_switch_fallback_transport(
        &mut self,
        session_telemetry: &SessionTelemetry,
        model_info: &ModelInfo,
    ) -> bool;

    async fn send_response_processed(&mut self, _response_id: &str) {}
}

/// Session-scoped factory for turn-scoped model runtimes.
pub trait ModelRuntime: Send + Sync {
    fn new_turn_runtime(&self) -> Box<dyn ModelTurnRuntime>;

    fn responses_websocket_enabled(&self) -> bool;

    fn current_window_id(&self) -> String;
}

#[derive(Clone)]
pub struct DefaultModelRuntime {
    client: ModelClient,
}

impl DefaultModelRuntime {
    pub fn new(client: ModelClient) -> Self {
        Self { client }
    }
}

impl ModelRuntime for DefaultModelRuntime {
    fn new_turn_runtime(&self) -> Box<dyn ModelTurnRuntime> {
        Box::new(self.client.new_session())
    }

    fn responses_websocket_enabled(&self) -> bool {
        self.client.responses_websocket_enabled()
    }

    fn current_window_id(&self) -> String {
        self.client.current_window_id()
    }
}

#[async_trait]
impl ModelTurnRuntime for ModelClientSession {
    async fn stream(&mut self, request: ModelStreamRequest<'_>) -> CodexResult<ResponseStream> {
        let stream = self
            .stream(
                request.prompt,
                request.model_info,
                request.session_telemetry,
                request.effort,
                request.summary,
                request.service_tier,
                request.turn_metadata_header,
                request.inference_trace,
            )
            .or_cancel(&request.cancellation_token)
            .await??;
        Ok(stream)
    }

    fn try_switch_fallback_transport(
        &mut self,
        session_telemetry: &SessionTelemetry,
        model_info: &ModelInfo,
    ) -> bool {
        self.try_switch_fallback_transport(session_telemetry, model_info)
    }

    async fn send_response_processed(&mut self, response_id: &str) {
        ModelClientSession::send_response_processed(self, response_id).await;
    }
}

const RUNTIME_IDLE_CHANNEL_CAPACITY: usize = 1024;

/// Embeddable Codex runtime host for durable runtimes that need direct access to
/// Codex's native thread and event lifecycle.
pub struct CodexRuntimeHost {
    config: Config,
    auth_manager: Arc<AuthManager>,
    thread_manager: Arc<ThreadManager>,
    idle_tx: broadcast::Sender<String>,
}

pub struct CodexRuntimeHostOptions {
    pub config: Config,
    pub session_source: SessionSource,
}

impl CodexRuntimeHostOptions {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            session_source: SessionSource::Custom("azoz-ai-runtime".to_string()),
        }
    }
}

impl CodexRuntimeHost {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        Self::with_options(CodexRuntimeHostOptions::new(config)).await
    }

    pub async fn with_options(options: CodexRuntimeHostOptions) -> anyhow::Result<Self> {
        let CodexRuntimeHostOptions {
            config,
            session_source,
        } = options;
        let state_db = crate::init_state_db(&config).await;
        let auth_manager =
            AuthManager::shared_from_config(&config, /*enable_codex_api_key_env*/ false).await;
        let runtime_paths =
            ExecServerRuntimePaths::from_optional_paths(Some(std::env::current_exe()?), None)?;
        let environment_manager =
            EnvironmentManager::from_codex_home(config.codex_home.clone(), Some(runtime_paths))
                .await?;
        let thread_store = thread_store_from_config(&config, state_db.clone());
        let installation_id = resolve_installation_id(&config.codex_home).await?;
        let (idle_tx, _) = broadcast::channel(RUNTIME_IDLE_CHANNEL_CAPACITY);
        let mut extension_builder = ExtensionRegistryBuilder::<Config>::new();
        extension_builder.thread_lifecycle_contributor(Arc::new(RuntimeIdleNotifier {
            idle_tx: idle_tx.clone(),
        }));
        let thread_manager = Arc::new(ThreadManager::new(
            &config,
            auth_manager.clone(),
            session_source,
            Arc::new(environment_manager),
            Arc::new(extension_builder.build()),
            /*analytics_events_client*/ None,
            thread_store,
            state_db,
            installation_id,
            /*attestation_provider*/ None,
        ));

        Ok(Self {
            config,
            auth_manager,
            thread_manager,
            idle_tx,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn thread_manager(&self) -> Arc<ThreadManager> {
        Arc::clone(&self.thread_manager)
    }

    pub async fn open_thread(
        &self,
        thread_id: Option<&str>,
        dynamic_tools: Vec<DynamicToolSpec>,
    ) -> CodexResult<CodexRuntimeThread> {
        match thread_id {
            Some(thread_id) => {
                let thread_id = ThreadId::from_string(thread_id)
                    .map_err(|err| CodexErr::InvalidRequest(format!("invalid thread id: {err}")))?;
                match self.thread_manager.get_thread(thread_id).await {
                    Ok(thread) => Ok(self.runtime_thread(thread_id, thread)),
                    Err(CodexErr::ThreadNotFound(_)) => {
                        let NewThread {
                            thread_id, thread, ..
                        } = self
                            .thread_manager
                            .resume_thread_by_id_with_tools(
                                self.config.clone(),
                                thread_id,
                                Arc::clone(&self.auth_manager),
                                dynamic_tools,
                                /*persist_extended_history*/ false,
                                /*parent_trace*/ None,
                            )
                            .await?;
                        Ok(self.runtime_thread(thread_id, thread))
                    }
                    Err(err) => Err(err),
                }
            }
            None => {
                let NewThread {
                    thread_id, thread, ..
                } = self
                    .thread_manager
                    .start_thread_with_tools(
                        self.config.clone(),
                        dynamic_tools,
                        /*persist_extended_history*/ false,
                    )
                    .await?;
                Ok(self.runtime_thread(thread_id, thread))
            }
        }
    }

    fn runtime_thread(&self, thread_id: ThreadId, thread: Arc<CodexThread>) -> CodexRuntimeThread {
        CodexRuntimeThread {
            thread_id,
            thread,
            idle_rx: self.idle_tx.subscribe(),
            pending_idle_status: None,
        }
    }
}

pub struct CodexRuntimeThread {
    thread_id: ThreadId,
    thread: Arc<CodexThread>,
    idle_rx: broadcast::Receiver<String>,
    pending_idle_status: Option<AgentStatus>,
}

pub enum CodexRuntimeLoopItem {
    Event(Event),
    ThreadIdle {
        thread_id: ThreadId,
        status: AgentStatus,
    },
}

impl CodexRuntimeThread {
    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    pub fn thread_id_string(&self) -> String {
        self.thread_id.to_string()
    }

    pub async fn start_user_turn(
        &self,
        input_text: String,
        client_user_message_id: Option<String>,
    ) -> CodexResult<String> {
        self.thread
            .submit_user_input_with_client_user_message_id(
                user_text_op(input_text),
                /*trace*/ None,
                client_user_message_id,
            )
            .await
    }

    pub async fn steer_user_turn(
        &self,
        input_text: String,
        expected_turn_id: Option<&str>,
        client_user_message_id: Option<String>,
    ) -> CodexResult<String> {
        self.thread
            .steer_input(
                vec![user_text_input(input_text)],
                BTreeMap::new(),
                expected_turn_id,
                client_user_message_id,
                /*responsesapi_client_metadata*/ None,
            )
            .await
            .map_err(|err| CodexErr::InvalidRequest(format!("{err:?}")))
    }

    pub async fn submit_dynamic_tool_response(
        &self,
        call_id: String,
        response: DynamicToolResponse,
    ) -> CodexResult<String> {
        self.thread
            .submit(Op::DynamicToolResponse {
                id: call_id,
                response,
            })
            .await
    }

    pub async fn set_goal_for_turn(
        &self,
        turn_id: &str,
        objective: String,
        status: ThreadGoalStatus,
        tool_namespace: Option<String>,
    ) -> CodexResult<ThreadGoal> {
        let turn_context = self
            .thread
            .codex
            .session
            .turn_context_for_sub_id(turn_id)
            .await
            .ok_or_else(|| {
                CodexErr::InvalidRequest(format!("no active turn found for id {turn_id}"))
            })?;
        self.thread
            .codex
            .session
            .set_thread_goal(
                &turn_context,
                SetGoalRequest {
                    objective: Some(objective),
                    status: Some(status),
                    tool_namespace,
                    token_budget: None,
                },
            )
            .await
            .map_err(|err| CodexErr::InvalidRequest(err.to_string()))
    }

    pub async fn next_event_or_idle(&mut self) -> CodexResult<CodexRuntimeLoopItem> {
        loop {
            if let Some(event) = self.thread.try_next_event()? {
                return Ok(CodexRuntimeLoopItem::Event(event));
            }
            if let Some(status) = self.pending_idle_status.take() {
                return Ok(CodexRuntimeLoopItem::ThreadIdle {
                    thread_id: self.thread_id,
                    status,
                });
            }
            tokio::select! {
                event = self.thread.next_event() => {
                    return event.map(CodexRuntimeLoopItem::Event);
                }
                idle = self.idle_rx.recv() => {
                    match idle {
                        Ok(thread_id) if thread_id == self.thread_id.to_string() => {
                            let status = self.thread.agent_status().await;
                            if let Some(event) = self.thread.try_next_event()? {
                                self.pending_idle_status = Some(status);
                                return Ok(CodexRuntimeLoopItem::Event(event));
                            }
                            return Ok(CodexRuntimeLoopItem::ThreadIdle {
                                thread_id: self.thread_id,
                                status,
                            });
                        }
                        Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            return Err(CodexErr::InternalAgentDied);
                        }
                    }
                }
            }
        }
    }

    pub async fn agent_status(&self) -> AgentStatus {
        self.thread.agent_status().await
    }
}

fn user_text_op(input_text: String) -> Op {
    Op::UserInput {
        items: vec![user_text_input(input_text)],
        environments: Some(Vec::new()),
        final_output_json_schema: None,
        responsesapi_client_metadata: None,
        additional_context: BTreeMap::new(),
        thread_settings: Default::default(),
    }
}

fn user_text_input(input_text: String) -> UserInput {
    UserInput::Text {
        text: input_text,
        text_elements: Vec::new(),
    }
}

struct RuntimeIdleNotifier {
    idle_tx: broadcast::Sender<String>,
}

#[async_trait]
impl ThreadLifecycleContributor<Config> for RuntimeIdleNotifier {
    async fn on_thread_idle(&self, input: ThreadIdleInput<'_>) {
        let _ = self.idle_tx.send(input.thread_store.level_id().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_user_text_op_explicitly_disables_turn_environments() {
        let Op::UserInput { environments, .. } = user_text_op("hello".to_string()) else {
            panic!("expected user input op");
        };
        assert_eq!(environments, Some(Vec::new()));
    }
}
