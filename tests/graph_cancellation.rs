use tokio_util::sync::CancellationToken;
use weaver_rs::prelude::*;

#[tokio::test]
async fn pre_cancelled_token_aborts_run_before_any_node_executes() {
    let graph = Graph::<u32>::builder()
        .add_node("a".into(), |ctx| {
            Box::pin(async move {
                ctx.state += 1;
                Ok(NodeId::end())
            })
        })
        .add_edge(NodeId::start(), "a".into())
        .add_edge("a".into(), NodeId::end())
        .build()
        .unwrap();

    let token = CancellationToken::new();
    token.cancel();

    let result = SingleAgentRuntime::new(graph)
        .with_cancellation(token)
        .run(0_u32)
        .await;

    match result {
        Err(LoomError::Cancelled) => {}
        other => panic!("expected Cancelled, got {other:?}"),
    }
}
