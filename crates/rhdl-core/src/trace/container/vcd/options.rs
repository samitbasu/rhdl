//! Options for VCD File generation

/// Options to control the VCD File generation.
pub struct VcdOptions {
    /// Tail time - amount of time after the last event to continue the trace
    pub tail_flush_time: u64,
}

impl Default for VcdOptions {
    fn default() -> Self {
        Self {
            tail_flush_time: 100,
        }
    }
}
