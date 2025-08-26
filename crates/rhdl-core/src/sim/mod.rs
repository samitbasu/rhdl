pub mod clock_pos_edge;
pub mod merge;
pub mod probe;
pub mod reset;
pub mod run;
pub mod test_module;
pub mod testbench;
pub mod vcd;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ResetOrData<T> {
    Reset,
    Data(T),
}
