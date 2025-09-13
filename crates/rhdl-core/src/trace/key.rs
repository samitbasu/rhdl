use std::hash::Hash;

pub trait TraceKey: Clone + Copy + Hash {
    fn as_string(&self) -> String;
}

impl TraceKey for &'static str {
    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl TraceKey for usize {
    fn as_string(&self) -> String {
        format!("{self}")
    }
}

impl TraceKey for &[&'static str] {
    fn as_string(&self) -> String {
        self.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl<T: TraceKey, U: TraceKey> TraceKey for (T, U) {
    fn as_string(&self) -> String {
        format!("{}.{}", self.0.as_string(), self.1.as_string())
    }
}
