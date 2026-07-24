use crate::function_tool::FunctionCallError;
use crate::goals::GoalRuntimeEvent;
use crate::goals::SetGoalRequest;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::goal_spec::UPDATE_GOAL_TOOL_NAME;
use crate::tools::handlers::goal_spec::create_update_goal_tool;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_protocol::protocol::ThreadGoalStatus;
use codex_tools::ToolName;
use codex_tools::ToolSpec;

use super::UpdateGoalArgs;
use super::goal_response;
use super::procedure_read_error;
use super::procedure_update_error;

pub struct UpdateGoalHandler;

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for UpdateGoalHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(UPDATE_GOAL_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_update_goal_tool()
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            session,
            turn,
            payload,
            ..
        } = invocation;

        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "update_procedure_status received an unsupported payload".to_string(),
                ));
            }
        };

        let args: UpdateGoalArgs = parse_arguments(&arguments)?;
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
        let current_procedure = session
            .get_thread_goal()
            .await
            .map_err(procedure_read_error)?;
        if !current_procedure.as_ref().is_some_and(|procedure| {
            matches!(
                procedure.status,
                ThreadGoalStatus::Active | ThreadGoalStatus::Paused
            )
        }) {
            return Err(FunctionCallError::RespondToModel(
                "No current procedure is active. Activate a published procedure before updating its status."
                    .to_string(),
            ));
        }
        session
            .goal_runtime_apply(GoalRuntimeEvent::ToolCompletedGoal {
                turn_context: turn.as_ref(),
            })
            .await
            .map_err(procedure_update_error)?;
        let goal = session
            .set_thread_goal(
                turn.as_ref(),
                SetGoalRequest {
                    objective: None,
                    status: Some(args.status),
                    tool_namespace: None,
                    token_budget: None,
                },
            )
            .await
            .map_err(procedure_update_error)?;
        goal_response(Some(goal)).map(boxed_tool_output)
    }
}

impl CoreToolRuntime for UpdateGoalHandler {}
