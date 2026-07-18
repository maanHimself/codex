use super::ContextualUserFragment;

const CONTEXT_START_MARKER: &str = "<azoz_procedure_context>";
const CONTEXT_END_MARKER: &str = "</azoz_procedure_context>";

/// Runtime-owned customer-service procedure context supplied to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AzozProcedureContext {
    body: String,
}

impl AzozProcedureContext {
    pub fn new(body: impl Into<String>) -> Self {
        Self { body: body.into() }
    }
}

impl ContextualUserFragment for AzozProcedureContext {
    fn role() -> &'static str {
        "user"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        (CONTEXT_START_MARKER, CONTEXT_END_MARKER)
    }

    fn body(&self) -> String {
        format!("\n{}\n", self.body)
    }
}
