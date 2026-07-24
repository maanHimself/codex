//! Built-in model tool handlers backed by persisted thread goals.
//!
//! Customer-service sessions expose the persisted state to the model as the
//! current procedure. Procedure activation is host-controlled; the model can
//! only read the current procedure and update its status.

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use codex_protocol::protocol::ThreadGoal;
use codex_protocol::protocol::ThreadGoalStatus;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Write as _;

mod get_goal;
mod update_goal;

pub use get_goal::GetGoalHandler;
pub use update_goal::UpdateGoalHandler;

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

impl ProcedureToolResponse {
    fn new(goal: Option<ThreadGoal>) -> Self {
        Self {
            procedure: goal
                .filter(|goal| {
                    matches!(
                        goal.status,
                        ThreadGoalStatus::Active | ThreadGoalStatus::Paused
                    )
                })
                .map(|goal| ProcedureToolState {
                    objective: goal.objective,
                    status: goal.status,
                }),
        }
    }
}

fn format_goal_error(err: anyhow::Error) -> String {
    let mut message = err.to_string();
    for cause in err.chain().skip(1) {
        let _ = write!(message, ": {cause}");
    }
    message
}

fn goal_response(goal: Option<ThreadGoal>) -> Result<FunctionToolOutput, FunctionCallError> {
    let response = serde_json::to_string_pretty(&ProcedureToolResponse::new(goal))
        .map_err(|err| FunctionCallError::Fatal(err.to_string()))?;
    Ok(FunctionToolOutput::from_text(response, Some(true)))
}

fn procedure_read_error(err: anyhow::Error) -> FunctionCallError {
    procedure_state_error(
        "read",
        "The current procedure state could not be read because the procedure runtime failed. Do not assume that a procedure is active.",
        err,
    )
}

fn procedure_update_error(err: anyhow::Error) -> FunctionCallError {
    procedure_state_error(
        "update",
        "The current procedure status could not be updated because the procedure runtime rejected the change. Do not assume that the requested status was applied.",
        err,
    )
}

fn procedure_state_error(
    operation: &'static str,
    model_message: &'static str,
    err: anyhow::Error,
) -> FunctionCallError {
    tracing::error!(
        operation,
        error = %format_goal_error(err),
        "procedure state operation failed"
    );
    FunctionCallError::RespondToModel(model_message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::ThreadId;
    use pretty_assertions::assert_eq;

    #[test]
    fn terminal_procedure_response_is_not_current() {
        let goal = ThreadGoal {
            thread_id: ThreadId::new(),
            objective: "Keep optimizing".to_string(),
            status: ThreadGoalStatus::Complete,
            tool_namespace: None,
            token_budget: Some(10_000),
            tokens_used: 3_250,
            time_used_seconds: 75,
            created_at: 1,
            updated_at: 2,
        };

        let response = ProcedureToolResponse::new(Some(goal));

        assert_eq!(response, ProcedureToolResponse { procedure: None });
    }

    #[test]
    fn active_procedure_response_exposes_only_objective_and_status() {
        let goal = ThreadGoal {
            thread_id: ThreadId::new(),
            objective: "Keep optimizing".to_string(),
            status: ThreadGoalStatus::Active,
            tool_namespace: None,
            token_budget: Some(10_000),
            tokens_used: 3_250,
            time_used_seconds: 75,
            created_at: 1,
            updated_at: 2,
        };

        let response = ProcedureToolResponse::new(Some(goal));

        assert_eq!(
            response,
            ProcedureToolResponse {
                procedure: Some(ProcedureToolState {
                    objective: "Keep optimizing".to_string(),
                    status: ThreadGoalStatus::Active,
                }),
            }
        );
    }

    #[test]
    fn missing_procedure_response_is_null() {
        let response = ProcedureToolResponse::new(None);

        assert_eq!(response, ProcedureToolResponse { procedure: None });
    }
}
