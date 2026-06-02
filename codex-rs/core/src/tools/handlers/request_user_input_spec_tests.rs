use super::*;
use codex_tools::JsonSchema;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn request_user_input_tool_includes_question_schema() {
    assert_eq!(
        create_request_user_input_tool("Ask the user to choose.".to_string()),
        ToolSpec::Function(ResponsesApiTool {
            name: "request_user_input".to_string(),
            description: "Ask the user to choose.".to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(BTreeMap::from([(
                    "question".to_string(),
                    JsonSchema::string(Some(
                        "Question to ask the customer as a normal assistant message."
                            .to_string(),
                    )),
                )]), Some(vec!["question".to_string()]), Some(false.into())),
            output_schema: None,
        })
    );
}

#[test]
fn request_user_input_tool_description_is_plain() {
    assert_eq!(
        request_user_input_tool_description(),
        "Ask the customer a question as a normal assistant message.".to_string()
    );
}
