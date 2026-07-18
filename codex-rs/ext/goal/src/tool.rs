use std::sync::Arc;

use async_trait::async_trait;
use codex_extension_api::FunctionCallError;
use codex_extension_api::JsonToolOutput;
use codex_extension_api::ToolCall;
use codex_extension_api::ToolExecutor;
use codex_extension_api::ToolName;
use codex_extension_api::ToolOutput;
use codex_extension_api::ToolSpec;
use codex_protocol::ThreadId;
use codex_protocol::protocol::ThreadGoal;
use codex_protocol::protocol::ThreadGoalStatus;
use serde::Deserialize;
use serde::Serialize;

use crate::accounting::BudgetLimitedGoalDisposition;
use crate::accounting::GoalAccountingState;
use crate::events::GoalEventEmitter;
use crate::metrics::GoalMetrics;
use crate::spec::GET_GOAL_TOOL_NAME;
use crate::spec::UPDATE_GOAL_TOOL_NAME;
use crate::spec::create_get_goal_tool;
use crate::spec::create_update_goal_tool;

#[derive(Clone)]
pub(crate) struct GoalToolExecutor {
    kind: GoalToolKind,
    thread_id: ThreadId,
    state_db: Arc<codex_state::StateRuntime>,
    accounting_state: Arc<GoalAccountingState>,
    event_emitter: GoalEventEmitter,
    metrics: GoalMetrics,
}

#[derive(Clone, Copy)]
enum GoalToolKind {
    Get,
    Update,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct UpdateGoalArgs {
    status: ThreadGoalStatus,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcedureToolResponse {
    procedure: Option<ProcedureToolState>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcedureToolState {
    objective: String,
    status: ThreadGoalStatus,
}

impl GoalToolExecutor {
    pub(crate) fn get(
        thread_id: ThreadId,
        state_db: Arc<codex_state::StateRuntime>,
        accounting_state: Arc<GoalAccountingState>,
        event_emitter: GoalEventEmitter,
        metrics: GoalMetrics,
    ) -> Self {
        Self {
            kind: GoalToolKind::Get,
            thread_id,
            state_db,
            accounting_state,
            event_emitter,
            metrics,
        }
    }

    pub(crate) fn update(
        thread_id: ThreadId,
        state_db: Arc<codex_state::StateRuntime>,
        accounting_state: Arc<GoalAccountingState>,
        event_emitter: GoalEventEmitter,
        metrics: GoalMetrics,
    ) -> Self {
        Self {
            kind: GoalToolKind::Update,
            thread_id,
            state_db,
            accounting_state,
            event_emitter,
            metrics,
        }
    }
}

#[async_trait]
impl ToolExecutor<ToolCall> for GoalToolExecutor {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(match self.kind {
            GoalToolKind::Get => GET_GOAL_TOOL_NAME,
            GoalToolKind::Update => UPDATE_GOAL_TOOL_NAME,
        })
    }

    fn spec(&self) -> ToolSpec {
        match self.kind {
            GoalToolKind::Get => create_get_goal_tool(),
            GoalToolKind::Update => create_update_goal_tool(),
        }
    }

    async fn handle(&self, invocation: ToolCall) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        match self.kind {
            GoalToolKind::Get => self.handle_get(invocation).await,
            GoalToolKind::Update => self.handle_update(invocation).await,
        }
    }
}

impl GoalToolExecutor {
    async fn handle_get(
        &self,
        invocation: ToolCall,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let _ = invocation.function_arguments()?;
        let goal = self
            .state_db
            .thread_goals()
            .get_thread_goal(self.thread_id)
            .await
            .map(|goal| goal.map(protocol_goal_from_state))
            .map_err(procedure_read_error)?;
        procedure_response(goal)
    }

    async fn handle_update(
        &self,
        invocation: ToolCall,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let args: UpdateGoalArgs = parse_arguments(invocation.function_arguments()?)?;
        if !matches!(
            args.status,
            ThreadGoalStatus::Active
                | ThreadGoalStatus::Paused
                | ThreadGoalStatus::Complete
                | ThreadGoalStatus::Blocked
        ) {
            return Err(FunctionCallError::RespondToModel(
                "update_procedure_status accepts only active, paused, complete, or blocked"
                    .to_string(),
            ));
        }

        let previous_status = self
            .state_db
            .thread_goals()
            .get_thread_goal(self.thread_id)
            .await
            .map_err(procedure_read_error)?
            .map(|goal| goal.status)
            .ok_or_else(|| {
                FunctionCallError::RespondToModel(
                    "No current procedure is active. Activate a published procedure before updating its status."
                        .to_string(),
                )
            })?;
        if args.status != ThreadGoalStatus::Active {
            self.account_active_goal_progress(
                match args.status {
                    ThreadGoalStatus::Complete => codex_state::GoalAccountingMode::ActiveOrComplete,
                    ThreadGoalStatus::Paused | ThreadGoalStatus::Blocked => {
                        codex_state::GoalAccountingMode::ActiveOrStopped
                    }
                    ThreadGoalStatus::Active
                    | ThreadGoalStatus::UsageLimited
                    | ThreadGoalStatus::BudgetLimited => unreachable!("status validated above"),
                },
                invocation.call_id.as_str(),
                BudgetLimitedGoalDisposition::ClearActive,
            )
            .await?;
        }
        let goal = self
            .state_db
            .thread_goals()
            .update_thread_goal(
                self.thread_id,
                codex_state::GoalUpdate {
                    objective: None,
                    status: Some(state_status_from_protocol(args.status)),
                    token_budget: None,
                    expected_goal_id: None,
                },
            )
            .await
            .map_err(procedure_update_error)?
            .ok_or_else(|| {
                FunctionCallError::RespondToModel(
                    "The current procedure status could not be updated because the procedure no longer exists."
                        .to_string(),
                )
            })?;
        self.metrics
            .record_resumed_if_status_changed(Some(previous_status), goal.status);
        self.metrics
            .record_terminal_if_status_changed(Some(previous_status), &goal);
        let turn_id = if goal.status == codex_state::ThreadGoalStatus::Active {
            self.accounting_state
                .mark_current_turn_goal_active(goal.goal_id.clone())
        } else {
            self.accounting_state.clear_current_turn_goal()
        };
        let goal = protocol_goal_from_state(goal);
        self.emit_goal_updated_from_tool_call(&invocation, turn_id, goal.clone());
        procedure_response(Some(goal))
    }

    fn emit_goal_updated_from_tool_call(
        &self,
        invocation: &ToolCall,
        turn_id: Option<String>,
        goal: ThreadGoal,
    ) {
        self.event_emitter
            .thread_goal_updated(invocation.call_id.clone(), turn_id, goal);
    }

    async fn account_active_goal_progress(
        &self,
        mode: codex_state::GoalAccountingMode,
        event_id: &str,
        budget_limited_goal_disposition: BudgetLimitedGoalDisposition,
    ) -> Result<Option<ThreadGoal>, FunctionCallError> {
        let Some(turn_id) = self.accounting_state.current_turn_id() else {
            return Ok(None);
        };
        let Some(snapshot) = self.accounting_state.progress_snapshot(turn_id.as_str()) else {
            return Ok(None);
        };
        let previous_status = self
            .current_goal_status_for_metrics(Some(snapshot.expected_goal_id.as_str()))
            .await?;
        let outcome = self
            .state_db
            .thread_goals()
            .account_thread_goal_usage(
                self.thread_id,
                snapshot.time_delta_seconds,
                snapshot.token_delta,
                mode,
                Some(snapshot.expected_goal_id.as_str()),
            )
            .await
            .map_err(procedure_update_error)?;
        Ok(match outcome {
            codex_state::GoalAccountingOutcome::Updated(goal) => {
                self.metrics
                    .record_terminal_if_status_changed(previous_status, &goal);
                self.accounting_state.mark_progress_accounted_for_status(
                    turn_id.as_str(),
                    &snapshot,
                    goal.status,
                    budget_limited_goal_disposition,
                );
                let goal = protocol_goal_from_state(goal);
                self.event_emitter.thread_goal_updated(
                    event_id.to_string(),
                    Some(turn_id),
                    goal.clone(),
                );
                Some(goal)
            }
            codex_state::GoalAccountingOutcome::Unchanged(_) => None,
        })
    }

    async fn current_goal_status_for_metrics(
        &self,
        expected_goal_id: Option<&str>,
    ) -> Result<Option<codex_state::ThreadGoalStatus>, FunctionCallError> {
        let goal = self
            .state_db
            .thread_goals()
            .get_thread_goal(self.thread_id)
            .await
            .map_err(procedure_read_error)?;
        Ok(goal.and_then(|goal| {
            expected_goal_id
                .is_none_or(|expected_goal_id| goal.goal_id == expected_goal_id)
                .then_some(goal.status)
        }))
    }
}

fn parse_arguments<T>(arguments: &str) -> Result<T, FunctionCallError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(arguments)
        .map_err(|err| FunctionCallError::RespondToModel(err.to_string()))
}

fn procedure_response(goal: Option<ThreadGoal>) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
    let value = serde_json::to_value(ProcedureToolResponse::new(goal))
        .map_err(|err| FunctionCallError::Fatal(err.to_string()))?;
    Ok(Box::new(JsonToolOutput::new(value)))
}

impl ProcedureToolResponse {
    fn new(goal: Option<ThreadGoal>) -> Self {
        Self {
            procedure: goal.map(|goal| ProcedureToolState {
                objective: goal.objective,
                status: goal.status,
            }),
        }
    }
}

fn procedure_read_error(err: impl std::fmt::Display) -> FunctionCallError {
    procedure_state_error(
        "read",
        "The current procedure state could not be read because the procedure runtime failed. Do not assume that a procedure is active.",
        err,
    )
}

fn procedure_update_error(err: impl std::fmt::Display) -> FunctionCallError {
    procedure_state_error(
        "update",
        "The current procedure status could not be updated because the procedure runtime rejected the change. Do not assume that the requested status was applied.",
        err,
    )
}

fn procedure_state_error(
    operation: &'static str,
    model_message: &'static str,
    err: impl std::fmt::Display,
) -> FunctionCallError {
    tracing::error!(operation, error = %err, "procedure state operation failed");
    FunctionCallError::RespondToModel(model_message.to_string())
}

pub(crate) fn protocol_goal_from_state(goal: codex_state::ThreadGoal) -> ThreadGoal {
    ThreadGoal {
        thread_id: goal.thread_id,
        objective: goal.objective,
        status: protocol_status_from_state(goal.status),
        tool_namespace: goal.tool_namespace,
        token_budget: goal.token_budget,
        tokens_used: goal.tokens_used,
        time_used_seconds: goal.time_used_seconds,
        created_at: goal.created_at.timestamp(),
        updated_at: goal.updated_at.timestamp(),
    }
}

fn protocol_status_from_state(status: codex_state::ThreadGoalStatus) -> ThreadGoalStatus {
    match status {
        codex_state::ThreadGoalStatus::Active => ThreadGoalStatus::Active,
        codex_state::ThreadGoalStatus::Paused => ThreadGoalStatus::Paused,
        codex_state::ThreadGoalStatus::Blocked => ThreadGoalStatus::Blocked,
        codex_state::ThreadGoalStatus::UsageLimited => ThreadGoalStatus::UsageLimited,
        codex_state::ThreadGoalStatus::BudgetLimited => ThreadGoalStatus::BudgetLimited,
        codex_state::ThreadGoalStatus::Complete => ThreadGoalStatus::Complete,
    }
}

fn state_status_from_protocol(status: ThreadGoalStatus) -> codex_state::ThreadGoalStatus {
    match status {
        ThreadGoalStatus::Active => codex_state::ThreadGoalStatus::Active,
        ThreadGoalStatus::Paused => codex_state::ThreadGoalStatus::Paused,
        ThreadGoalStatus::Blocked => codex_state::ThreadGoalStatus::Blocked,
        ThreadGoalStatus::UsageLimited => codex_state::ThreadGoalStatus::UsageLimited,
        ThreadGoalStatus::BudgetLimited => codex_state::ThreadGoalStatus::BudgetLimited,
        ThreadGoalStatus::Complete => codex_state::ThreadGoalStatus::Complete,
    }
}
