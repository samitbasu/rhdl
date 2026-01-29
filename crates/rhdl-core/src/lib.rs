//#![warn(missing_docs)]
#![deny(unsafe_code)]
//#![deny(clippy::unwrap_used)]
//#![deny(clippy::expect_used)]
//#![deny(clippy::panic)]
#![deny(unused_must_use)]
pub use types::kind::Kind;
pub mod clock_details;

pub use circuit::circuit_impl::Circuit;
pub use circuit::circuit_impl::CircuitDQ;
pub use circuit::circuit_impl::CircuitIO;
pub use circuit::descriptor::AsyncKind;
pub use circuit::descriptor::Descriptor;
pub use circuit::descriptor::SyncKind;
pub use circuit::hdl_descriptor::HDLDescriptor;
pub use circuit::synchronous::Synchronous;
pub use circuit::synchronous::SynchronousDQ;
pub use circuit::synchronous::SynchronousIO;
pub use clock_details::ClockDetails;
//pub use types::bitz::BitZ;
pub use types::clock::Clock;
pub use types::digital::Digital;
pub use types::digital_fn::DigitalFn;
pub use types::digital_fn::DigitalFn2;
pub use types::digital_fn::DigitalFn3;
pub use types::domain::Color;
pub use types::domain::Domain;
pub use types::kernel::KernelFnKind;
pub use types::kind::DiscriminantAlignment;
pub use types::reset::Reset;
pub use types::reset_n::ResetN;
pub use types::signal::Signal;
pub use types::timed::Timed;
pub mod ast;
pub mod circuit;
pub mod compiler;
pub mod types;
pub mod util;
pub use util::id;

pub use compiler::compile_design;
pub use trace::key::TraceKey;
pub use trace::page::trace;
pub use trace::page::trace_pop_path;
pub use trace::page::trace_push_path;
pub use types::kind::DiscriminantType;
pub use types::typed_bits::TypedBits;
pub mod rhif;
pub use ast::builder;
pub use types::clock;
pub use types::digital_fn;
pub use types::digital_fn::DigitalSignature;
pub use types::kernel;

pub const MAX_ITERS: usize = 10;
pub mod error;
pub use error::RHDLError;
pub mod rtl;
pub use compiler::CompilationMode;
pub use types::clock_reset::ClockReset;
pub use types::clock_reset::clock_reset;

pub mod sim;
pub use types::timed_sample::TimedSample;
pub use types::timed_sample::timed_sample;
pub mod hdl;
pub use bitx::dyn_bit_manip::move_nbits_to_msb;
pub use rhdl_trace_type::TraceType;
pub use trace::rtt;
pub mod bitx;
pub use bitx::BitX;
pub use bitx::bitx_vec;
pub mod common;
pub mod ntl;
pub use circuit::scoped_name::ScopedName;
pub mod trace;
