// traits/checkpoint.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub run_id:      String,
    pub node_id:     String,
    pub state_json:  serde_json::Value,   // erased — runtime restores with T
    pub interrupted: Option<InterruptReason>,
    pub saved_at:    DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterruptReason {
    HumanApproval { prompt: String },
    ExternalEvent  { event_id: String },
    Scheduled      { resume_at: DateTime<Utc> },
}

#[async_trait]
pub trait CheckpointStore: Send + Sync {
    async fn save(&self, checkpoint: &Checkpoint) -> Result<(), LoomError>;
    async fn load(&self, run_id: &str) -> Result<Option<Checkpoint>, LoomError>;
    async fn delete(&self, run_id: &str) -> Result<(), LoomError>;
}
