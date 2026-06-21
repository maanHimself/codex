use super::ContextualUserFragment;
use codex_protocol::prompt_overrides;
use codex_protocol::protocol::REALTIME_CONVERSATION_CLOSE_TAG;
use codex_protocol::protocol::REALTIME_CONVERSATION_OPEN_TAG;

const REALTIME_END_INSTRUCTIONS: &str = include_str!("prompts/realtime/realtime_end.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RealtimeEndInstructions {
    reason: String,
}

impl RealtimeEndInstructions {
    pub(crate) fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl ContextualUserFragment for RealtimeEndInstructions {
    fn role() -> &'static str {
        "developer"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        (
            REALTIME_CONVERSATION_OPEN_TAG,
            REALTIME_CONVERSATION_CLOSE_TAG,
        )
    }

    fn body(&self) -> String {
        let instructions = prompt_overrides::resolve_prompt_str(
            prompt_overrides::REALTIME_END_INSTRUCTIONS,
            REALTIME_END_INSTRUCTIONS,
        );
        format!("\n{}\n\nReason: {}\n", instructions.trim(), self.reason)
    }
}
