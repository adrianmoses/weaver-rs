use schemars::JsonSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::primitives::state::AgentState;

pub trait StructuredOutput:
    Serialize + DeserializeOwned + JsonSchema + Clone + Send + Sync + 'static
{
}

impl<T> StructuredOutput for T where
    T: Serialize + DeserializeOwned + JsonSchema + Clone + Send + Sync + 'static
{
}

pub trait GraphState: Serialize + DeserializeOwned + Send + Sync + Clone + 'static {}

impl<T: StructuredOutput> GraphState for AgentState<T> {}
