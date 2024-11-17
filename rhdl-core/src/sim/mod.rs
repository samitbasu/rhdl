use crate::Digital;

pub mod clock_pos_edge;
pub mod merge;
pub mod probe;
pub mod run;
pub mod run_synchronous;
pub mod stream;
pub mod waveform;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ResetOrData<T: Digital> {
    Reset,
    Data(T),
}
