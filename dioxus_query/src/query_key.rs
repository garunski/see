use std::fmt;
use tracing::trace;

/// Query key for identifying cached queries
///
/// Keys are composed of string parts joined by colons.
/// Example: `QueryKey::new(&["user", "123"])` creates "user:123"
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QueryKey(String);

impl QueryKey {
    /// Create a new query key from parts
    ///
    /// # Example
    /// ```
    /// use s_e_e_dioxus_query::prelude::QueryKey;
    /// let key = QueryKey::new(&["user", "123"]);
    /// assert_eq!(key.to_string(), "user:123");
    /// ```
    pub fn new(parts: &[&str]) -> Self {
        let key = parts.join(":");
        trace!(key = %key, "Creating new QueryKey");
        Self(key)
    }

    /// Create a query key from a string directly
    pub fn from_string(s: String) -> Self {
        trace!(key = %s, "Creating QueryKey from string");
        Self(s)
    }

    /// Get the string representation of the key
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for QueryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
