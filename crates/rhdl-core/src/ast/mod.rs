#![warn(missing_docs)]
//#![warn(clippy::missing_docs_in_private_items)]
//! RHDL Abstract Syntax Tree (AST) modules and types.
//!
//! This module contains the core definitions for the RHDL AST,
//! including nodes, expressions, statements, types, and related
//! constructs.  It also includes utilities for building and
//! manipulating the AST, as well as source location tracking.
pub(crate) mod ast_impl;
pub mod builder;
pub(crate) mod visit;
pub use ast_impl::KernelFlags;
pub(crate) mod spanned_source;
pub(crate) use ast_impl::SourceLocation;
pub(crate) use spanned_source::SourcePool;
pub(crate) use spanned_source::SpannedSource;
