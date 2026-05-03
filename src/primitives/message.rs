// primitives/message.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role:    Role,
    pub content: MessageContent,
    pub meta:    Option<MessageMeta>,  // token counts, model, latency
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role { System, User, Assistant, Tool }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    ToolCalls(Vec<ToolCall>),
    ToolResult { call_id: String, content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMeta {
    pub input_tokens:  Option<u32>,
    pub output_tokens: Option<u32>,
    pub latency_ms:    Option<u64>,
    pub model:         Option<String>,
}
