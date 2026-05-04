use std::sync::Arc;

use weaver_rs::prelude::*;

#[derive(Clone)]
struct State {
    flag: bool,
    visited: Vec<&'static str>,
}

#[tokio::test]
async fn conditional_edges_route_based_on_state_predicate_true_branch() {
    let final_state = run_with_flag(true).await;
    assert_eq!(final_state.visited, vec!["router", "left"]);
}

#[tokio::test]
async fn conditional_edges_route_based_on_state_predicate_false_branch() {
    let final_state = run_with_flag(false).await;
    assert_eq!(final_state.visited, vec!["router", "right"]);
}

async fn run_with_flag(flag: bool) -> State {
    let if_flag: EdgeCondition<State> = Arc::new(|s: &State| s.flag);
    let if_not_flag: EdgeCondition<State> = Arc::new(|s: &State| !s.flag);

    let graph = Graph::<State>::builder()
        .add_node("router".into(), |ctx| {
            Box::pin(async move {
                ctx.state.visited.push("router");
                Ok("router".into())
            })
        })
        .add_node("left".into(), |ctx| {
            Box::pin(async move {
                ctx.state.visited.push("left");
                Ok(NodeId::end())
            })
        })
        .add_node("right".into(), |ctx| {
            Box::pin(async move {
                ctx.state.visited.push("right");
                Ok(NodeId::end())
            })
        })
        .add_edge(NodeId::start(), "router".into())
        .add_conditional_edge("router".into(), "left".into(), if_flag)
        .add_conditional_edge("router".into(), "right".into(), if_not_flag)
        .add_edge("left".into(), NodeId::end())
        .add_edge("right".into(), NodeId::end())
        .build()
        .unwrap();

    SingleAgentRuntime::new(graph)
        .run(State {
            flag,
            visited: Vec::new(),
        })
        .await
        .unwrap()
}
