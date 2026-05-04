//! Smallest possible runnable graph — the canonical "hello world" for weaver-rs.
//!
//! Run with: `cargo run --example graph_basic`

use weaver_rs::prelude::*;

#[derive(Default, Debug)]
struct AppState {
    greeting: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let graph = Graph::<AppState>::builder()
        .add_node("greet".into(), |ctx| {
            Box::pin(async move {
                ctx.state.greeting = Some("hello, weaver-rs".into());
                Ok(NodeId::end())
            })
        })
        .add_edge(NodeId::start(), "greet".into())
        .add_edge("greet".into(), NodeId::end())
        .build()?;

    let final_state = SingleAgentRuntime::new(graph).run(AppState::default()).await?;

    println!("final state: {final_state:?}");
    Ok(())
}
