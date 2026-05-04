use std::collections::HashMap;

use crate::graph::edge::Edge;
use crate::graph::node::{NodeFn, NodeId};

pub struct Graph<S> {
    pub(crate) nodes: HashMap<NodeId, NodeFn<S>>,
    pub(crate) edges: Vec<Edge<S>>,
}

impl<S> Graph<S> {
    pub fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes.keys()
    }

    pub fn has_node(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id) || id.is_start() || id.is_end()
    }

    pub(crate) fn node_fn(&self, id: &NodeId) -> Option<&NodeFn<S>> {
        self.nodes.get(id)
    }

    pub(crate) fn next_node(&self, from: &NodeId, state: &S) -> Option<NodeId> {
        self.edges
            .iter()
            .filter(|e| &e.from == from)
            .find(|e| e.matches(state))
            .map(|e| e.to.clone())
    }
}
