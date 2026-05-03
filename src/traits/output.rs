// traits/output.rs

// The two bounds that flow through everything in Loom
pub trait StructuredOutput: 
    Serialize + DeserializeOwned + JsonSchema + Send + Sync + 'static {}

impl<T> StructuredOutput for T 
where T: Serialize + DeserializeOwned + JsonSchema + Send + Sync + 'static {}

// GraphState adds checkpoint-ability — required for interrupt/resume
pub trait GraphState: 
    Serialize + DeserializeOwned + Send + Sync + Clone + 'static {}

impl<T: StructuredOutput> GraphState for AgentState<T> {}
