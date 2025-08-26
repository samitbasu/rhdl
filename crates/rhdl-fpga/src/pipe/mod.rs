#![warn(missing_docs)]
//! Pipe Cores
//!
//! Pipe cores are used to build pipelines in which
//! strobed data elements flow and are transformed
//! as they proceed through the design.  They are
//! composable like iterators, and are meant to run
//! in high performance designs.  The pipe cores all have
//! registered inputs, so that they can be chained to build
//! pipelines that do not combinatorial pathways between stages.
//!
//! Note that Pipe Cores do _not_ provide a means of supplying
//! backpressure.  If a data element is present at the entrance
//! of the pipeline, it _must_ be accepted and processed.  
//! If you need a pipeline that supports backpressure then look
//! at the [crate::stream] module, which provides cores that
//! support backpressure.
//!
//! Pipe cores generally accept items of type `Option<T>`, and
//! return some other item of type `Option<S>`.  If the item is
//! a `Some` variant, it is considered valid.
pub mod chunked;
pub mod filter;
pub mod filter_map;
pub mod map;
