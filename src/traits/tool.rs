// traits/tool.rs

#[async_trait]
pub trait Tool: Send + Sync {
    fn schema(&self) -> ToolSchema;
    async fn call(&self, call: &ToolCall) -> Result<ToolResult, LoomError>;
}
