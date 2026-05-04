use tokio_util::sync::CancellationToken;

use crate::error::LoomError;
use crate::graph::ctx::AgentCtx;
use crate::graph::graph::Graph;
use crate::graph::node::NodeId;

const DEFAULT_ITERATION_CAP: usize = 25;

pub struct SingleAgentRuntime<S> {
    graph: Graph<S>,
    iteration_cap: usize,
    cancel: CancellationToken,
}

impl<S: Send + 'static> SingleAgentRuntime<S> {
    pub fn new(graph: Graph<S>) -> Self {
        Self {
            graph,
            iteration_cap: DEFAULT_ITERATION_CAP,
            cancel: CancellationToken::new(),
        }
    }

    pub fn with_iteration_cap(mut self, cap: usize) -> Self {
        self.iteration_cap = cap;
        self
    }

    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.cancel = token;
        self
    }

    pub async fn run(self, initial_state: S) -> Result<S, LoomError> {
        if self.cancel.is_cancelled() {
            return Err(LoomError::Cancelled);
        }

        let run_id = uuid::Uuid::new_v4().to_string();
        let mut ctx = AgentCtx::new(initial_state, run_id, self.cancel.clone());

        let mut current = self
            .graph
            .next_node(&NodeId::start(), &ctx.state)
            .ok_or_else(|| {
                LoomError::Graph("no edge from Start matched the initial state".into())
            })?;

        loop {
            if self.cancel.is_cancelled() {
                return Err(LoomError::Cancelled);
            }

            if current.is_end() {
                return Ok(ctx.state);
            }

            if ctx.step >= self.iteration_cap {
                return Err(LoomError::MaxIterations(self.iteration_cap));
            }

            let node_fn = self.graph.node_fn(&current).ok_or_else(|| {
                LoomError::Graph(format!("no registered node for id: {current}"))
            })?;

            ctx.run.node_id = current.to_string();
            let returned = node_fn(&mut ctx).await?;
            ctx.step += 1;
            ctx.run.step = ctx.step;

            current = if returned == current {
                self.graph.next_node(&current, &ctx.state).ok_or_else(|| {
                    LoomError::Graph(format!(
                        "node {current} returned itself but no outgoing edge matched the state"
                    ))
                })?
            } else if returned.is_end() || self.graph.has_node(&returned) {
                returned
            } else {
                return Err(LoomError::Graph(format!(
                    "node {current} returned unknown next id: {returned}"
                )));
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_iteration_cap_is_25() {
        let graph = Graph::<()>::builder()
            .add_node("a".into(), |_ctx| {
                Box::pin(async move { Ok(NodeId::end()) })
            })
            .add_edge(NodeId::start(), "a".into())
            .add_edge("a".into(), NodeId::end())
            .build()
            .unwrap();
        let runtime = SingleAgentRuntime::new(graph);
        assert_eq!(runtime.iteration_cap, DEFAULT_ITERATION_CAP);
    }

    #[test]
    fn with_iteration_cap_overrides() {
        let graph = Graph::<()>::builder()
            .add_node("a".into(), |_ctx| {
                Box::pin(async move { Ok(NodeId::end()) })
            })
            .add_edge(NodeId::start(), "a".into())
            .add_edge("a".into(), NodeId::end())
            .build()
            .unwrap();
        let runtime = SingleAgentRuntime::new(graph).with_iteration_cap(7);
        assert_eq!(runtime.iteration_cap, 7);
    }
}
