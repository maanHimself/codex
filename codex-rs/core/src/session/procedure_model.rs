use std::sync::Arc;

use codex_features::Feature;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::ThreadGoalStatus;
use tracing::error;
use tracing::info;

use super::Session;
use super::turn_context::TurnContext;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ActiveProcedureState {
    active: bool,
    tool_namespace: Option<String>,
}

impl ActiveProcedureState {
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn tool_namespace(&self) -> Option<String> {
        self.tool_namespace.clone()
    }
}

pub(crate) struct ProcedureModelContextSelection {
    pub(crate) context: Arc<TurnContext>,
    pub(crate) switched: bool,
}

pub(crate) async fn read_active_procedure_state(
    session: &Session,
) -> CodexResult<ActiveProcedureState> {
    if !session.enabled(Feature::Goals) {
        return Ok(ActiveProcedureState::default());
    }

    let goal = match session.get_thread_goal().await {
        Ok(goal) => goal,
        Err(err) => {
            error!(error = %err, "failed to read active procedure state");
            return Err(CodexErr::InternalServerError);
        }
    };
    Ok(match goal {
        Some(goal) if goal.status == ThreadGoalStatus::Active => ActiveProcedureState {
            active: true,
            tool_namespace: goal.tool_namespace,
        },
        _ => ActiveProcedureState::default(),
    })
}

pub(crate) async fn procedure_model_was_selected_for_turn(
    session: &Session,
    turn_context: &TurnContext,
) -> bool {
    let Some(procedure_model) = turn_context.config.procedure_model.as_deref() else {
        return false;
    };
    session.reference_context_item().await.is_some_and(|item| {
        item.turn_id.as_deref() == Some(turn_context.sub_id.as_str())
            && item.model == procedure_model
    })
}

pub(crate) async fn select_procedure_model_context(
    session: &Session,
    turn_context: Arc<TurnContext>,
) -> CodexResult<ProcedureModelContextSelection> {
    let Some(procedure_model) = turn_context.config.procedure_model.clone() else {
        return Ok(ProcedureModelContextSelection {
            context: turn_context,
            switched: false,
        });
    };
    let procedure_reasoning_effort = turn_context.config.procedure_model_reasoning_effort;

    if turn_context.model_info.slug == procedure_model
        && (procedure_reasoning_effort.is_none()
            || turn_context.reasoning_effort == procedure_reasoning_effort)
    {
        if turn_context.model_info.used_fallback_model_metadata {
            error!(
                model = %procedure_model,
                "configured procedure model is missing from the model catalog"
            );
            return Err(CodexErr::InternalServerError);
        }
        return Ok(ProcedureModelContextSelection {
            context: turn_context,
            switched: false,
        });
    }

    let next_context = Arc::new(
        turn_context
            .with_model_and_reasoning_effort(
                procedure_model.clone(),
                procedure_reasoning_effort,
                &session.services.models_manager,
            )
            .await,
    );
    if next_context.model_info.used_fallback_model_metadata {
        error!(
            model = %procedure_model,
            "configured procedure model is missing from the model catalog"
        );
        return Err(CodexErr::InternalServerError);
    }

    info!(
        turn_id = %turn_context.sub_id,
        from_model = %turn_context.model_info.slug,
        to_model = %next_context.model_info.slug,
        from_reasoning_effort = ?turn_context.reasoning_effort,
        to_reasoning_effort = ?next_context.reasoning_effort,
        reason = "active_procedure",
        "switching model for procedure sampling"
    );
    Ok(ProcedureModelContextSelection {
        context: next_context,
        switched: true,
    })
}
