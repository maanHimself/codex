//! Responses API tool definitions for persisted thread goals.
//!
//! These specs expose goal read/update primitives to the model while keeping
//! usage accounting system-managed.

use codex_protocol::prompt_overrides;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use serde_json::json;
use std::collections::BTreeMap;

pub const GET_GOAL_TOOL_NAME: &str = "get_goal";
pub const CREATE_GOAL_TOOL_NAME: &str = "create_goal";
pub const UPDATE_GOAL_TOOL_NAME: &str = "update_goal";

pub fn create_get_goal_tool() -> ToolSpec {
    ToolSpec::Function(ResponsesApiTool {
        name: GET_GOAL_TOOL_NAME.to_string(),
        description: prompt_overrides::resolve_prompt(
            prompt_overrides::GET_GOAL_TOOL_DESCRIPTION,
            "Get the current goal for this thread, including status, budgets, token and elapsed-time usage, and remaining token budget.",
        ),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(BTreeMap::new(), Some(Vec::new()), Some(false.into())),
        output_schema: None,
    })
}

pub fn create_create_goal_tool() -> ToolSpec {
    let properties = BTreeMap::from([
        (
            "objective".to_string(),
            JsonSchema::string(Some(
                "Required. The concrete objective to start pursuing. This starts a new active goal only when no goal is currently defined; if a goal already exists, this tool fails."
                    .to_string(),
            )),
        ),
        (
            "token_budget".to_string(),
            JsonSchema::integer(Some(
                "Positive token budget for the new goal. Omit unless explicitly requested."
                    .to_string(),
            )),
        ),
    ]);
    let built_in_description = format!(
        r#"Create a goal only when explicitly requested by the user or system/developer instructions; do not infer goals from ordinary tasks.
Set token_budget only when an explicit token budget is requested. Fails if a goal exists; use {UPDATE_GOAL_TOOL_NAME} only for status."#
    );

    ToolSpec::Function(ResponsesApiTool {
        name: CREATE_GOAL_TOOL_NAME.to_string(),
        description: prompt_overrides::resolve_prompt(
            prompt_overrides::CREATE_GOAL_TOOL_DESCRIPTION,
            &built_in_description,
        ),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            /*required*/ Some(vec!["objective".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

pub fn create_update_goal_tool() -> ToolSpec {
    let properties = BTreeMap::from([(
        "status".to_string(),
        JsonSchema::string_enum(
            vec![
                json!("active"),
                json!("paused"),
                json!("complete"),
                json!("blocked"),
            ],
            Some(
                "Required. Set to `active` to resume a paused procedure, `paused` when the customer asks to stop, defer, or switch away, `complete` only when the procedure is actually complete, and `blocked` only for a real impasse. Do not set budget_limited or usage_limited; those statuses are system controlled."
                    .to_string(),
            ),
        ),
    )]);

    ToolSpec::Function(ResponsesApiTool {
        name: UPDATE_GOAL_TOOL_NAME.to_string(),
        description: prompt_overrides::resolve_prompt(
            prompt_overrides::UPDATE_GOAL_TOOL_DESCRIPTION,
            r#"Update the existing goal status for a running customer procedure.
Use `active` only to resume a paused procedure.
Use `paused` when the customer asks to stop, continue later, or switch away before the procedure is complete.
Use `complete` only when the authored procedure is actually complete and no required work remains.
Use `blocked` only for a real impasse that cannot be resolved by asking the customer or using available tools.
Do not use `blocked` merely because the work is hard, slow, uncertain, incomplete, or would benefit from clarification.
Do not mark a goal complete merely because its budget is nearly exhausted or because you are stopping work.
You cannot set budget_limited or usage_limited; those statuses are controlled by the system.
When marking a budgeted goal achieved with status `complete`, report the final token usage from the tool result to the user. Do not report usage for unbudgeted goals."#,
        ),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            /*required*/ Some(vec!["status".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_goal_tool_exposes_model_managed_procedure_statuses() {
        let ToolSpec::Function(tool) = create_update_goal_tool() else {
            panic!("update_goal should be a function tool");
        };
        let status = tool
            .parameters
            .properties
            .as_ref()
            .and_then(|properties| properties.get("status"))
            .expect("status property should exist");

        assert_eq!(
            status.enum_values,
            Some(vec![
                json!("active"),
                json!("paused"),
                json!("complete"),
                json!("blocked"),
            ])
        );
    }
}
