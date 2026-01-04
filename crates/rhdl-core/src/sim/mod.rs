//! Built-in Simulator and Testbench support
#![warn(missing_docs)]
pub mod iter;
pub mod probe;
pub mod run;
pub mod test_module;
pub mod testbench;
//pub mod vcd;

/// An enum representing either a reset signal or data.
///
/// `ResetOrData::Reset` indicates a reset pulse, while
/// `ResetOrData::Data(T)` carries actual data of type `T`.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ResetOrData<T> {
    /// A reset pulse.
    Reset,
    /// Actual data of type `T`.
    Data(T),
}

/// Extension traits to provide easy access to simulation utilities with iterators.
pub mod extension {
    pub use crate::sim::iter::clock_pos_edge::ClockPosEdgeExt;
    pub use crate::sim::iter::merge_map::MergeMapExt;
    pub use crate::sim::iter::reset::ResetExt;
    pub use crate::sim::iter::uniform::UniformExt;
}
