pub mod meta;
pub mod page;
pub mod record;
pub mod trace_tree;
pub mod traceable;

/// A unique identifier for a traced value across all
/// pages in a simulation session.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TraceId(u64);

impl std::hash::Hash for TraceId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl nohash::IsEnabled for TraceId {}
