use codex_core::context::AzozProcedureContext;
use codex_core::context::ContextualUserFragment;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::ThreadGoal;

const BUDGET_LIMIT_PROMPT_TEMPLATE: &str =
    include_str!("../../../core/templates/goals/budget_limit.md");
const OBJECTIVE_UPDATED_PROMPT_TEMPLATE: &str =
    include_str!("../../../core/templates/goals/objective_updated.md");

pub(crate) fn budget_limit_steering_item(goal: &ThreadGoal) -> ResponseItem {
    goal_context_input_item(budget_limit_prompt(goal))
}

pub(crate) fn objective_updated_steering_item(goal: &ThreadGoal) -> ResponseItem {
    goal_context_input_item(objective_updated_prompt(goal))
}

fn goal_context_input_item(prompt: String) -> ResponseItem {
    ContextualUserFragment::into(AzozProcedureContext::new(prompt))
}

fn budget_limit_prompt(goal: &ThreadGoal) -> String {
    let objective = escape_xml_text(&goal.objective);
    BUDGET_LIMIT_PROMPT_TEMPLATE.replace("{{ objective }}", objective.as_str())
}

fn objective_updated_prompt(goal: &ThreadGoal) -> String {
    let objective = escape_xml_text(&goal.objective);
    OBJECTIVE_UPDATED_PROMPT_TEMPLATE.replace("{{ objective }}", objective.as_str())
}

fn escape_xml_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
