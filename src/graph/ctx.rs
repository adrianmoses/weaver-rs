use chrono::Utc;
use tokio_util::sync::CancellationToken;

use crate::primitives::state::RunMeta;

pub struct AgentCtx<S> {
    pub state: S,
    pub run: RunMeta,
    pub step: usize,
    pub cancel: CancellationToken,
}

impl<S> AgentCtx<S> {
    pub fn new(state: S, run_id: impl Into<String>, cancel: CancellationToken) -> Self {
        Self {
            state,
            run: RunMeta {
                run_id: run_id.into(),
                started_at: Utc::now(),
                node_id: String::new(),
                step: 0,
            },
            step: 0,
            cancel,
        }
    }
}
