use weaver_rs::prelude::*;

#[test]
fn build_rejects_graph_with_no_start_edge() {
    let result: Result<Graph<()>> = Graph::<()>::builder()
        .add_node("a".into(), |_ctx| {
            Box::pin(async move { Ok(NodeId::end()) })
        })
        .build();

    match result {
        Err(LoomError::Graph(msg)) => assert!(msg.contains("Start"), "msg: {msg}"),
        Err(e) => panic!("expected Graph error mentioning Start, got {e:?}"),
        Ok(_) => panic!("expected Graph error, got Ok(graph)"),
    }
}

#[test]
fn build_rejects_edge_to_unknown_node() {
    let result: Result<Graph<()>> = Graph::<()>::builder()
        .add_node("a".into(), |_ctx| {
            Box::pin(async move { Ok(NodeId::end()) })
        })
        .add_edge(NodeId::start(), "ghost".into())
        .build();

    match result {
        Err(LoomError::Graph(msg)) => assert!(msg.contains("unknown"), "msg: {msg}"),
        Err(e) => panic!("expected Graph error mentioning unknown node, got {e:?}"),
        Ok(_) => panic!("expected Graph error, got Ok(graph)"),
    }
}

#[test]
fn build_rejects_unreachable_node() {
    let result: Result<Graph<()>> = Graph::<()>::builder()
        .add_node("a".into(), |_ctx| {
            Box::pin(async move { Ok(NodeId::end()) })
        })
        .add_node("orphan".into(), |_ctx| {
            Box::pin(async move { Ok(NodeId::end()) })
        })
        .add_edge(NodeId::start(), "a".into())
        .add_edge("a".into(), NodeId::end())
        .build();

    match result {
        Err(LoomError::Graph(msg)) => assert!(msg.contains("unreachable"), "msg: {msg}"),
        Err(e) => panic!("expected Graph error mentioning unreachable, got {e:?}"),
        Ok(_) => panic!("expected Graph error, got Ok(graph)"),
    }
}
