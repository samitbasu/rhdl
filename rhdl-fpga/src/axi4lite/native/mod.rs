//! Native for AXI
//!
//! In general, the AXI protocol can be pretty complicated to manage, since
//! the various signals are presented along different (latency insensitive)
//! channels, and the the protocol management can be fairly complicated.
//!
//! These cores provide wrappers that allow you to interface native cores
//! into the AXI interface, on either the read side or the write side (or both).
pub mod read;
pub mod write;
