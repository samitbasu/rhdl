//! Scoped Name
//!
//! A scoped name represents a hierarchical name in the circuit design,
//! where each component of the name corresponds to a level in the hierarchy.
//! For example, a scoped name `top.module1.submodule2` would represent
//! a submodule named `submodule2` inside a module named `module1`, which
//! is itself inside the top-level module named `top`.

/// A hierarchical scoped name for circuit components.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedName(Vec<String>);

impl ScopedName {
    /// Create a new scoped name that represents the top-level scope.
    #[must_use]
    pub fn top() -> Self {
        Self(vec!["top".into()])
    }
    /// Add a new component to the scoped name.
    pub fn push<S: Into<String>>(&mut self, name: S) {
        self.0.push(name.into());
    }
    /// Remove the last component from the scoped name.
    pub fn pop(&mut self) {
        self.0.pop();
    }
    /// Get the last component of the scoped name, if any.
    pub fn last(&self) -> Option<&String> {
        self.0.last()
    }
    /// Create a new scoped name by appending a component to the current one.
    #[must_use]
    pub fn with(&self, name: impl Into<String>) -> Self {
        let mut new = self.0.clone();
        new.push(name.into());
        Self(new)
    }
}

/// Convert a string slice into a scoped name.
impl From<&str> for ScopedName {
    fn from(s: &str) -> Self {
        Self(vec![s.to_string()])
    }
}

impl std::fmt::Display for ScopedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("_"))
    }
}
