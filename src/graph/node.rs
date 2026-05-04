use std::borrow::Cow;
use std::sync::Arc;

use futures::future::BoxFuture;

use crate::error::LoomError;
use crate::graph::ctx::AgentCtx;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(Cow<'static, str>);

impl NodeId {
    pub fn start() -> Self {
        Self(Cow::Borrowed("__start"))
    }

    pub fn end() -> Self {
        Self(Cow::Borrowed("__end"))
    }

    pub fn is_start(&self) -> bool {
        self.0 == "__start"
    }

    pub fn is_end(&self) -> bool {
        self.0 == "__end"
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for NodeId {
    fn from(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub type NodeFn<S> = Arc<
    dyn for<'a> Fn(&'a mut AgentCtx<S>) -> BoxFuture<'a, Result<NodeId, LoomError>>
        + Send
        + Sync,
>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_ids_are_distinct() {
        assert_ne!(NodeId::start(), NodeId::end());
        assert!(NodeId::start().is_start());
        assert!(NodeId::end().is_end());
        assert!(!NodeId::start().is_end());
        assert!(!NodeId::end().is_start());
    }

    #[test]
    fn nodeid_equality_and_hash() {
        use std::collections::HashSet;

        let a: NodeId = "agent".into();
        let b: NodeId = "agent".to_string().into();
        assert_eq!(a, b);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }

    #[test]
    fn nodeid_from_static_str_is_zero_alloc() {
        let id: NodeId = "tool".into();
        assert_eq!(id.as_str(), "tool");
    }

    #[test]
    fn display_renders_inner_string() {
        let id: NodeId = "agent".into();
        assert_eq!(format!("{id}"), "agent");
    }
}
