use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::handlers::request_user_input_spec::REQUEST_USER_INPUT_TOOL_NAME;
use crate::tools::handlers::request_user_input_spec::create_request_user_input_tool;
use crate::tools::handlers::request_user_input_spec::request_user_input_tool_description;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_tools::ToolName;
use codex_tools::ToolSpec;

pub struct RequestUserInputHandler;

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for RequestUserInputHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(REQUEST_USER_INPUT_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_request_user_input_tool(request_user_input_tool_description())
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let _ = invocation;
        Err(FunctionCallError::Fatal(format!(
            "{REQUEST_USER_INPUT_TOOL_NAME} reached runtime dispatch instead of output handling"
        )))
    }
}

impl CoreToolRuntime for RequestUserInputHandler {}

#[cfg(test)]
#[path = "request_user_input_tests.rs"]
mod tests;
