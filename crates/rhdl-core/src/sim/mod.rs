pub mod iter;
pub mod probe;
pub mod run;
pub mod test_module;
pub mod testbench;
pub mod vcd;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ResetOrData<T> {
    Reset,
    Data(T),
}

pub mod extension {
    pub use crate::sim::iter::clock_pos_edge::ClockPosEdgeExt;
    pub use crate::sim::iter::merge::MergeExt;
    pub use crate::sim::iter::reset::TimedStreamExt;
    pub use crate::sim::iter::uniform::UniformExt;
}
