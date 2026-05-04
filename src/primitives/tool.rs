use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub content: ResultContent,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResultContent {
    Text(String),
    Structured(serde_json::Value),
}

impl ToolResult {
    pub fn text(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            content: ResultContent::Text(content.into()),
            is_error: false,
        }
    }

    pub fn error(call_id: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            content: ResultContent::Text(msg.into()),
            is_error: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
