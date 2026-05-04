use async_trait::async_trait;

use crate::error::LoomError;
use crate::primitives::tool::{ToolCall, ToolResult, ToolSchema};

#[async_trait]
pub trait Tool: Send + Sync {
    fn schema(&self) -> ToolSchema;
    async fn call(&self, call: &ToolCall) -> Result<ToolResult, LoomError>;
}
