// traits/memory.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id:        String,
    pub content:   String,
    pub embedding: Option<Vec<f32>>,
    pub meta:      serde_json::Value,
    pub stored_at: DateTime<Utc>,
}

#[async_trait]
pub trait WorkingMemory: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, LoomError>;
    async fn set(&self, key: &str, value: serde_json::Value) -> Result<(), LoomError>;
    async fn delete(&self, key: &str) -> Result<(), LoomError>;
}

#[async_trait]
pub trait SemanticMemory: Send + Sync {
    async fn recall(&self, query: &str, top_k: usize) -> Result<Vec<MemoryEntry>, LoomError>;
    async fn store(&self, entry: MemoryEntry) -> Result<(), LoomError>;
}

#[async_trait]
pub trait EpisodicMemory: Send + Sync {
    async fn record(&self, run_id: &str, summary: &str, meta: serde_json::Value) -> Result<(), LoomError>;
    async fn recall(&self, run_id: &str) -> Result<Option<MemoryEntry>, LoomError>;
}
