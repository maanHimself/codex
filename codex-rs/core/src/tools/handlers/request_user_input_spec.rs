use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use std::collections::BTreeMap;

pub const REQUEST_USER_INPUT_TOOL_NAME: &str = "request_user_input";

pub fn create_request_user_input_tool(description: String) -> ToolSpec {
    let properties = BTreeMap::from([(
        "question".to_string(),
        JsonSchema::string(Some(
            "Question to ask the customer as a normal assistant message.".to_string(),
        )),
    )]);

    ToolSpec::Function(ResponsesApiTool {
        name: REQUEST_USER_INPUT_TOOL_NAME.to_string(),
        description,
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["question".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

pub fn request_user_input_tool_description() -> String {
    "Ask the customer a question as a normal assistant message.".to_string()
}

#[cfg(test)]
#[path = "request_user_input_spec_tests.rs"]
mod tests;
