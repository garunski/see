use std::fmt;
use tracing::trace;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QueryKey(String);

impl QueryKey {
    pub fn new(parts: &[&str]) -> Self {
        let key = parts.join(":");
        trace!(key = %key, "Creating new QueryKey");
        Self(key)
    }

    pub fn from_string(s: String) -> Self {
        trace!(key = %s, "Creating QueryKey from string");
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for QueryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
