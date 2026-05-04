use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use futures::future::BoxFuture;

use crate::error::LoomError;
use crate::graph::ctx::AgentCtx;
use crate::graph::edge::{Edge, EdgeCondition};
use crate::graph::graph::Graph;
use crate::graph::node::{NodeFn, NodeId};

pub struct GraphBuilder<S> {
    nodes: HashMap<NodeId, NodeFn<S>>,
    edges: Vec<Edge<S>>,
}

impl<S> Default for GraphBuilder<S> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
}

impl<S: Send + 'static> GraphBuilder<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node<F>(mut self, id: NodeId, f: F) -> Self
    where
        F: for<'a> Fn(&'a mut AgentCtx<S>) -> BoxFuture<'a, Result<NodeId, LoomError>>
            + Send
            + Sync
            + 'static,
    {
        let boxed: NodeFn<S> = Arc::new(f);
        self.nodes.insert(id, boxed);
        self
    }

    pub fn add_edge(mut self, from: NodeId, to: NodeId) -> Self {
        self.edges.push(Edge::unconditional(from, to));
        self
    }

    pub fn add_conditional_edge(mut self, from: NodeId, to: NodeId, when: EdgeCondition<S>) -> Self {
        self.edges.push(Edge::conditional(from, to, when));
        self
    }

    pub fn build(self) -> Result<Graph<S>, LoomError> {
        let start = NodeId::start();
        let end = NodeId::end();

        if !self.edges.iter().any(|e| e.from == start) {
            return Err(LoomError::Graph(
                "graph has no outgoing edge from Start".into(),
            ));
        }

        for edge in &self.edges {
            if edge.from != start && !self.nodes.contains_key(&edge.from) {
                return Err(LoomError::Graph(format!(
                    "edge originates from unknown node: {}",
                    edge.from
                )));
            }
            if edge.to != end && !self.nodes.contains_key(&edge.to) {
                return Err(LoomError::Graph(format!(
                    "edge points to unknown node: {}",
                    edge.to
                )));
            }
        }

        let mut reachable: HashSet<NodeId> = HashSet::new();
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        queue.push_back(start.clone());
        while let Some(id) = queue.pop_front() {
            if !reachable.insert(id.clone()) {
                continue;
            }
            for edge in self.edges.iter().filter(|e| e.from == id) {
                if !reachable.contains(&edge.to) {
                    queue.push_back(edge.to.clone());
                }
            }
        }

        for node in self.nodes.keys() {
            if !reachable.contains(node) {
                return Err(LoomError::Graph(format!(
                    "node is unreachable from Start: {node}"
                )));
            }
        }

        Ok(Graph {
            nodes: self.nodes,
            edges: self.edges,
        })
    }
}

impl<S: Send + 'static> Graph<S> {
    pub fn builder() -> GraphBuilder<S> {
        GraphBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop_node<S: Send + 'static>() -> impl for<'a> Fn(&'a mut AgentCtx<S>) -> BoxFuture<'a, Result<NodeId, LoomError>>
           + Send
           + Sync
           + Copy {
        |_ctx: &mut AgentCtx<S>| Box::pin(async move { Ok(NodeId::end()) }) as BoxFuture<'_, _>
    }

    #[test]
    fn build_fails_when_no_start_edge() {
        let result: Result<Graph<()>, _> = GraphBuilder::<()>::new()
            .add_node("a".into(), noop_node())
            .build();
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("expected build error, got Ok"),
        };
        assert!(matches!(err, LoomError::Graph(ref m) if m.contains("Start")));
    }

    #[test]
    fn build_fails_when_edge_references_unknown_node() {
        let result: Result<Graph<()>, _> = GraphBuilder::<()>::new()
            .add_node("a".into(), noop_node())
            .add_edge(NodeId::start(), "missing".into())
            .build();
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("expected build error, got Ok"),
        };
        assert!(matches!(err, LoomError::Graph(ref m) if m.contains("unknown")));
    }

    #[test]
    fn build_fails_when_node_is_unreachable() {
        let result: Result<Graph<()>, _> = GraphBuilder::<()>::new()
            .add_node("a".into(), noop_node())
            .add_node("orphan".into(), noop_node())
            .add_edge(NodeId::start(), "a".into())
            .add_edge("a".into(), NodeId::end())
            .build();
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("expected build error, got Ok"),
        };
        assert!(matches!(err, LoomError::Graph(ref m) if m.contains("unreachable")));
    }

    #[test]
    fn build_succeeds_for_minimal_valid_graph() {
        let graph: Graph<()> = GraphBuilder::<()>::new()
            .add_node("a".into(), noop_node())
            .add_edge(NodeId::start(), "a".into())
            .add_edge("a".into(), NodeId::end())
            .build()
            .expect("valid graph should build");
        assert!(graph.has_node(&"a".into()));
    }
}
