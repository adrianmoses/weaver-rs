// primitives/state.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState<T: StructuredOutput> {
    pub messages:   Vec<Message>,
    pub output:     Option<T>,
    pub reflection: ReflectionState,
    pub run:        RunMeta,
    pub scratch:    serde_json::Value,   // untyped working space for nodes
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReflectionState {
    pub iteration:     usize,
    pub last_critique: Option<String>,
    pub accepted:      bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMeta {
    pub run_id:     String,
    pub started_at: DateTime<Utc>,
    pub node_id:    String,        // current node — updated by runtime
    pub step:       usize,         // total steps taken across all nodes
}

impl<T: StructuredOutput> AgentState<T> {
    pub fn new(run_id: impl Into<String>, initial_message: Message) -> Self {
        Self {
            messages:   vec![initial_message],
            output:     None,
            reflection: ReflectionState::default(),
            run:        RunMeta {
                run_id:     run_id.into(),
                started_at: Utc::now(),
                node_id:    String::new(),
                step:       0,
            },
            scratch: serde_json::Value::Null,
        }
    }

    pub fn with_user_message(content: impl Into<String>) -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            Message { role: Role::User, content: MessageContent::Text(content.into()), meta: None }
        )
    }
}
