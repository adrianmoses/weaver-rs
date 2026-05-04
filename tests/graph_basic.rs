use weaver_rs::prelude::*;

#[derive(Default)]
struct State {
    counter: u32,
}

#[tokio::test]
async fn basic_graph_runs_start_to_end_and_returns_final_state() {
    let graph = Graph::<State>::builder()
        .add_node("agent".into(), |ctx| {
            Box::pin(async move {
                ctx.state.counter += 1;
                Ok(NodeId::end())
            })
        })
        .add_edge(NodeId::start(), "agent".into())
        .add_edge("agent".into(), NodeId::end())
        .build()
        .expect("graph builds");

    let final_state = SingleAgentRuntime::new(graph)
        .run(State::default())
        .await
        .expect("run succeeds");

    assert_eq!(final_state.counter, 1);
}

#[tokio::test]
async fn multi_node_path_visits_each_node_in_order() {
    let graph = Graph::<Vec<&'static str>>::builder()
        .add_node("a".into(), |ctx| {
            Box::pin(async move {
                ctx.state.push("a");
                Ok("b".into())
            })
        })
        .add_node("b".into(), |ctx| {
            Box::pin(async move {
                ctx.state.push("b");
                Ok("c".into())
            })
        })
        .add_node("c".into(), |ctx| {
            Box::pin(async move {
                ctx.state.push("c");
                Ok(NodeId::end())
            })
        })
        .add_edge(NodeId::start(), "a".into())
        .add_edge("a".into(), "b".into())
        .add_edge("b".into(), "c".into())
        .add_edge("c".into(), NodeId::end())
        .build()
        .unwrap();

    let final_state = SingleAgentRuntime::new(graph).run(Vec::new()).await.unwrap();

    assert_eq!(final_state, vec!["a", "b", "c"]);
}
