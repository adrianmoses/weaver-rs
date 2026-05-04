use std::sync::Arc;

use crate::graph::node::NodeId;

pub type EdgeCondition<S> = Arc<dyn Fn(&S) -> bool + Send + Sync>;

pub struct Edge<S> {
    pub from: NodeId,
    pub to: NodeId,
    pub when: Option<EdgeCondition<S>>,
}

impl<S> Edge<S> {
    pub fn unconditional(from: NodeId, to: NodeId) -> Self {
        Self {
            from,
            to,
            when: None,
        }
    }

    pub fn conditional(from: NodeId, to: NodeId, when: EdgeCondition<S>) -> Self {
        Self {
            from,
            to,
            when: Some(when),
        }
    }

    pub fn matches(&self, state: &S) -> bool {
        match &self.when {
            None => true,
            Some(p) => p(state),
        }
    }
}
