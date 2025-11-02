#![warn(missing_docs)]
//! Circuit-related modules and types.
//!
//! This module contains the core abstractions and implementations
//! for defining and working with hardware circuits in RHDL. It includes
//! traits for circuits, descriptors for circuit metadata, and HDL
//! generation capabilities.  It also includes various useful circuit
//! abstractions like adapters (which allow you to plug [Synchronous](crate::Synchronous)
//! circuits into [Circuit](crate::Circuit) contexts), arrays of circuits,
//! and chain circuits.
pub mod adapter;
pub mod array;
pub mod chain;
pub mod circuit_impl;
pub mod descriptor;
pub mod drc;
pub mod fixture;
pub mod function;
pub mod hdl;
pub mod hdl_descriptor;
pub mod phantom;
pub mod scoped_name;
pub mod synchronous;
