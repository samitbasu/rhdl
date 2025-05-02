//! FPGA Support for RHDL
#![warn(missing_docs)]
pub mod core;
pub mod fifo;
pub use anyhow::Result;
pub mod axi4lite;
pub mod cdc;
#[doc(hidden)]
pub mod doc;
pub mod dsp;
pub mod gearbox;
pub mod gray;
pub mod lid;
pub mod reset;
pub mod rng;
pub mod tristate;
