use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum LoomError {
    #[error("tool call failed: {tool} — {reason}")]
    ToolError { tool: String, reason: String },

    #[error("LLM call failed: {0}")]
    LlmError(String),

    #[error("structured output deserialization failed: {0}")]
    OutputDeserialize(#[from] serde_json::Error),

    #[error("checkpoint store error: {0}")]
    Checkpoint(String),

    #[error("memory error: {0}")]
    Memory(String),

    #[error("graph error: {0}")]
    Graph(String),

    #[error("max iterations reached: {0}")]
    MaxIterations(usize),

    #[error("cancelled")]
    Cancelled,

    #[error("interrupted: {0:?}")]
    Interrupted(InterruptReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterruptReason {
    HumanApproval { prompt: String },
    ExternalEvent { event_id: String },
    Scheduled { resume_at: DateTime<Utc> },
}

pub type Result<T> = std::result::Result<T, LoomError>;
