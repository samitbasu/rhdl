#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedName(Vec<String>);

impl ScopedName {
    pub fn top() -> Self {
        Self(vec!["top".into()])
    }
    pub fn push<S: Into<String>>(&mut self, name: S) {
        self.0.push(name.into());
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn last(&self) -> Option<&String> {
        self.0.last()
    }
    pub fn with(&self, name: impl Into<String>) -> Self {
        let mut new = self.0.clone();
        new.push(name.into());
        Self(new)
    }
}

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
