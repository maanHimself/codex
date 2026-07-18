//! Responses API tools for customer procedures backed by persisted thread goals.
//!
//! Procedure activation belongs to the host. The model can read the current
//! procedure and update its status, but it cannot create one.

use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use serde_json::json;
use std::collections::BTreeMap;

pub const GET_GOAL_TOOL_NAME: &str = "get_current_procedure";
pub const UPDATE_GOAL_TOOL_NAME: &str = "update_procedure_status";

pub fn create_get_goal_tool() -> ToolSpec {
    ToolSpec::Function(ResponsesApiTool {
        name: GET_GOAL_TOOL_NAME.to_string(),
        description: "Read the current customer-service procedure, including its authored objective and status. A procedure exists only after a published procedure tool activates it."
            .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(BTreeMap::new(), Some(Vec::new()), Some(false.into())),
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
                "Required. Set to `active` to resume a paused procedure, `paused` when the customer asks to stop, defer, or switch away, `complete` only when the procedure is actually complete, and `blocked` only for a real impasse."
                    .to_string(),
            ),
        ),
    )]);

    ToolSpec::Function(ResponsesApiTool {
        name: UPDATE_GOAL_TOOL_NAME.to_string(),
        description: r#"Update the status of the current customer-service procedure.
Use `active` only to resume a paused procedure.
Use `paused` when the customer asks to stop, continue later, or switch away before the procedure is complete.
Use `complete` only when the authored procedure is actually complete and no required work remains.
Use `blocked` only for a real impasse that cannot be resolved by asking the customer or using available tools.
Do not use `blocked` merely because the work is hard, slow, uncertain, incomplete, or would benefit from clarification.
Do not mark a procedure complete merely because you answered the customer or are stopping work."#
            .to_string(),
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
