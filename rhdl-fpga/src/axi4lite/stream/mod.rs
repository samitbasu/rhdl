/// AXI Stream Types
///
/// The AXI Stream is completely equivalent to the
/// `rhdl` Stream concept.  The only difference
/// is that the AXI Stream interface breaks
/// out the valid signal from the data to be processed.
///
/// This module provides some lightweight cores
/// (combinatorial only) to interface AXI streams
/// to `rhdl` streams and visa versa.
use rhdl::prelude::*;

pub mod axi_to_rhdl;
pub mod rhdl_to_axi;

pub mod sink;
pub mod source;
pub mod testing;

#[derive(PartialEq, Debug, Digital, Default)]
pub struct StreamMOSI<T: Digital + Default> {
    /// The data to be sent
    pub tdata: T,
    /// The data valid flag
    pub tvalid: bool,
}

#[derive(PartialEq, Debug, Digital, Default)]
pub struct StreamMISO {
    /// The ready flag from the consumer
    pub tready: bool,
}
