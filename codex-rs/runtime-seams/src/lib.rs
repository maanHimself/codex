use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::protocol::Event;
use serde_json::Value;
use uuid::Uuid;

/// Sink for Codex protocol events after core rollout persistence.
#[async_trait]
pub trait EventSink: Send + Sync {
    async fn record_event(&self, _event: &Event) -> CodexResult<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct NoopEventSink;

#[async_trait]
impl EventSink for NoopEventSink {}

/// Generator for protocol IDs that must become durable under replaying hosts.
pub trait IdGenerator: Send + Sync {
    fn submission_id(&self) -> String {
        Uuid::now_v7().to_string()
    }
}

#[derive(Debug, Default)]
pub struct DefaultIdGenerator;

impl IdGenerator for DefaultIdGenerator {}

#[derive(Clone, Debug)]
pub struct ToolExecutionRequest {
    pub call_id: String,
    pub tool_name: String,
    pub payload: Value,
}

pub type ToolExecutionFuture =
    Pin<Box<dyn Future<Output = CodexResult<ResponseInputItem>> + Send + 'static>>;

/// Wrapper around local tool execution.
pub trait ToolExecutionRuntime: Send + Sync {
    fn execute(
        &self,
        _request: ToolExecutionRequest,
        execute: ToolExecutionFuture,
    ) -> ToolExecutionFuture {
        execute
    }
}

#[derive(Debug, Default)]
pub struct DefaultToolExecutionRuntime;

impl ToolExecutionRuntime for DefaultToolExecutionRuntime {}
