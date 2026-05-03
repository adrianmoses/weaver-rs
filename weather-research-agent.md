The state type replaces MessagesState
```rust
rust#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WeatherReport {
    #[schemars(description = "The city and state, e.g., San Francisco, CA")]
    pub location: String,
    #[schemars(description = "Temperature in Fahrenheit")]
    pub temperature: i32,
    #[schemars(description = "Current weather conditions")]
    pub conditions: WeatherCondition,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WeatherCondition { Sunny, Cloudy, Rainy, Snowy }

// The graph state — typed, exhaustive, no stringly-typed keys
pub struct WeatherState {
    pub messages: Vec<Message>,
    pub output:   Option<WeatherReport>,  // None until the LLM is done tool-calling
}
```
output: Option<WeatherReport> replaces the final_output dict key. The compiler enforces its presence at every edge condition — no runtime KeyError equivalent.
The tool
```rust
rustpub struct GetWeatherTool;

#[async_trait]
impl Tool for GetWeatherTool {
    fn name(&self) -> &str { "get_weather" }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City and state, e.g. New York, NY"
                }
            },
            "required": ["location"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult> {
        let location = input["location"].as_str().unwrap_or("unknown");
        Ok(ToolResult::text(format!(
            "Current weather in {location} is sunny with a temperature of 72°F"
        )))
    }
}
```
No decorator magic, no separate ToolNode registration — the tool is a struct you pass to the registry once.
The full graph
```rust
rustuse std::sync::Arc;
use your_framework::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Tool registry — single source of truth for tool registration
    let registry = ToolRegistry::new()
        .register(Arc::new(GetWeatherTool));

    // 2. LLM client with structured output bound to WeatherReport
    let llm = LlmClient::openai("gpt-4o")
        .with_tools(registry.schemas());   // injects tool schemas into every call

    // 3. Agent node — equivalent to agent_node() in LangGraph
    let llm_ref = llm.clone();
    let registry_ref = registry.clone();

    let agent_node = move |ctx: &mut AgentCtx<WeatherState>| {
        let llm      = llm_ref.clone();
        let registry = registry_ref.clone();
        Box::pin(async move {
            // Single LLM call — returns either tool calls OR structured output
            let response = llm
                .chat_structured::<WeatherReport>(&ctx.state.messages)
                .await?;

            match response {
                // LLM wants to call a tool — dispatch and append result to messages
                LlmResponse::ToolCalls(calls) => {
                    let results = registry.dispatch_parallel(calls).await?;
                    ctx.state.messages.extend(results.into_messages());
                    Ok(NodeId::Agent)   // loop back
                }
                // LLM produced final structured output — we're done
                LlmResponse::Structured(report) => {
                    ctx.state.output = Some(report);
                    Ok(NodeId::End)
                }
            }
        })
    };

    // 4. Build the graph — no conditional edge lambda needed,
    //    routing is handled inside the node via the LlmResponse enum
    let graph = Graph::<WeatherState>::builder()
        .add_node(NodeId::Agent, agent_node)
        .add_edge(NodeId::Start, NodeId::Agent)
        // NodeId::End is a terminal — no outgoing edges needed
        .build()?;

    // 5. Run
    let initial_state = WeatherState {
        messages: vec![Message::user("What is the weather in New York?")],
        output:   None,
    };

    let result = SingleAgentRuntime::new(graph)
        .run(initial_state)
        .await?;

    // output is typed — no dict unpacking, no Option<Any>
    if let Some(report) = result.state.output {
        println!("{}: {}°F, {:?}", report.location, report.temperature, report.conditions);
    }

    Ok(())
}
```
