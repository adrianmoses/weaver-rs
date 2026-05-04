use weaver_rs::prelude::*;

#[tokio::test]
async fn iteration_cap_terminates_a_looping_graph() {
    let graph = Graph::<u32>::builder()
        .add_node("loop".into(), |ctx| {
            Box::pin(async move {
                ctx.state += 1;
                Ok("loop".into())
            })
        })
        .add_edge(NodeId::start(), "loop".into())
        .add_edge("loop".into(), "loop".into())
        .build()
        .unwrap();

    let result = SingleAgentRuntime::new(graph)
        .with_iteration_cap(3)
        .run(0_u32)
        .await;

    match result {
        Err(LoomError::MaxIterations(cap)) => assert_eq!(cap, 3),
        other => panic!("expected MaxIterations(3), got {other:?}"),
    }
}
