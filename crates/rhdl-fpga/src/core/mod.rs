#![warn(missing_docs)]
//! Foundation primitives: registers, RAMs, counters, constants,
//! arithmetic / combinational helpers, and small control-flow
//! widgets that compose into larger circuits.
//!
//! Anything in this module is meant to be a *building block* —
//! generic, reusable, with no off-chip-specific behaviour.
//! Protocol PHYs, video formatters, and other off-chip-facing
//! widgets live in dedicated category modules at the workspace
//! level (`serial_bus`, `video`, `axi4lite`, etc.).
pub mod barrel_shifter;
pub mod comparator;
pub mod constant;
pub mod counter;
pub mod crc;
pub mod debouncer;
pub mod delay;
pub mod dff;
pub mod divider;
pub mod edge_detector;
pub mod leading_zeros;
pub mod mac;
pub mod one_hot;
pub mod option;
pub mod popcount;
pub mod priority_encoder;
pub mod pulse_stretcher;
pub mod pwm;
pub mod ram;
pub mod register_file;
pub mod rle_decoder;
pub mod rle_encoder;
pub mod round_robin_arbiter;
pub mod slice;
pub mod strict_priority_arbiter;
