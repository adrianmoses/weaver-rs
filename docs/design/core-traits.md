
```rust
// State is just a type parameter — you own it entirely
pub struct Graph<S: Send + 'static> {
    nodes: HashMap<NodeId, BoxedNode<S>>,
    edges: Vec<(NodeId, NodeId, Option<EdgeCondition<S>>)>,
}

// A node is an async closure over mutable state
type BoxedNode<S> = Box<dyn Fn(&mut AgentCtx<S>) -> BoxFuture<'_, Result<NodeId>> + Send + Sync>;

// EdgeCondition is just a predicate — no magic
pub type EdgeCondition<S> = Arc<dyn Fn(&S) -> bool + Send + Sync>;

// The strategy trait — implement this for ReAct, ReWOO, etc.
#[async_trait]
pub trait AgentStrategy<S: Send>: Send + Sync {
    async fn step(&self, ctx: &mut AgentCtx<S>) -> Result<AgentAction>;
}

pub enum AgentAction {
    CallTools(Vec<ToolCall>),    // dispatch to tool registry
    Respond(String),             // emit to state
    Transition(NodeId),          // hand off to graph
    Done,
}

// Tools are self-describing — schema drives LLM structured calls
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> serde_json::Value;  // JSON Schema for the LLM
    async fn call(&self, input: serde_json::Value) -> Result<ToolResult>;
}
```

ReWOO / prefetch works because ToolCall is a struct you can plan ahead of execution — ReWOO has the LLM emit all tool calls upfront as a plan, then ToolRegistry::dispatch_parallel(calls) runs them concurrently with tokio::join_all. No observation loop needed.

LLMCompiler extends this by adding a dependency DAG over tool calls — call B only after A's result is available. A small topological sort over Vec<ToolCall> gives you the execution schedule. You can represent this as:
```rust
pub struct ToolCallPlan {
    calls: Vec<ToolCall>,
    deps: HashMap<ToolCallId, Vec<ToolCallId>>,  // DAG
}
```
Multi-agent is just MultiAgentRuntime holding a HashMap<AgentId, Box<dyn AgentStrategy<S>>> where each agent runs in its own tokio::task. The message bus (tokio::broadcast::Sender<AgentMessage>) lets them coordinate without sharing state directly.
Structured output is a compile-time constraint on S:
```rust
pub struct GraphBuilder<S: DeserializeOwned + JsonSchema + Send> { ... }
```
The JSON schema for S is injected into the system prompt automatically — the LLM writes into state.structured, and you deserialize at the graph boundary.
