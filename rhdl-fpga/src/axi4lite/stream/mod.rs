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
pub mod axi_to_rhdl;
pub mod rhdl_to_axi;
