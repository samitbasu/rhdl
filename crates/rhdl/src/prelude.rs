pub use rhdl_bits::BitWidth;
pub use rhdl_bits::Bits;
pub use rhdl_bits::SignedBits;
pub use rhdl_bits::alias::*;
pub use rhdl_bits::bits;
pub use rhdl_bits::signed;
pub use rhdl_core::CircuitDQ;
pub use rhdl_core::ClockReset;
pub use rhdl_core::CompilationMode;
pub use rhdl_core::TraceKey;
pub use rhdl_core::circuit::adapter::Adapter;
pub use rhdl_core::circuit::circuit_descriptor::CircuitDescriptor;
pub use rhdl_core::circuit::circuit_impl::Circuit;
pub use rhdl_core::circuit::circuit_impl::CircuitIO;
pub use rhdl_core::circuit::func::Func;
pub use rhdl_core::circuit::hdl_descriptor::HDLDescriptor;
pub use rhdl_core::circuit::synchronous::Synchronous;
pub use rhdl_core::circuit::synchronous::SynchronousDQ;
pub use rhdl_core::circuit::synchronous::SynchronousIO;
pub use rhdl_core::compile_design;
pub use rhdl_core::compiler::driver::compile_design_stage1;
pub use rhdl_core::error::RHDLError;
pub use rhdl_core::rhif::spec::OpCode;
pub use rhdl_core::rtl::Object;
pub use rhdl_core::rtl::vm::execute;
pub use rhdl_core::trace;
pub use rhdl_core::trace::db::with_trace_db;
pub use rhdl_core::trace_init_db;
pub use rhdl_core::trace_pop_path;
pub use rhdl_core::trace_push_path;
pub use rhdl_core::trace_time;
pub use rhdl_core::types::bitz::BitZ;
pub use rhdl_core::types::clock::Clock;
pub use rhdl_core::types::clock::clock;
pub use rhdl_core::types::clock_reset::clock_reset;
pub use rhdl_core::types::digital::Digital;
pub use rhdl_core::types::digital_fn::DigitalFn;
pub use rhdl_core::types::digital_fn::DigitalFn0;
pub use rhdl_core::types::digital_fn::DigitalFn1;
pub use rhdl_core::types::digital_fn::DigitalFn2;
pub use rhdl_core::types::digital_fn::DigitalFn3;
pub use rhdl_core::types::digital_fn::DigitalFn4;
pub use rhdl_core::types::digital_fn::DigitalFn5;
pub use rhdl_core::types::digital_fn::DigitalFn6;
pub use rhdl_core::types::digital_fn::NoKernel2;
pub use rhdl_core::types::digital_fn::NoKernel3;
pub use rhdl_core::types::domain::Domain;
pub use rhdl_core::types::domain::{Blue, Green, Indigo, Orange, Red, Violet, Yellow};
pub use rhdl_core::types::kind::Kind;
pub use rhdl_core::types::path::Path;
pub use rhdl_core::types::path::bit_range;
pub use rhdl_core::types::reset::Reset;
pub use rhdl_core::types::reset::reset;
pub use rhdl_core::types::reset_n::ResetN;
pub use rhdl_core::types::reset_n::reset_n;
pub use rhdl_core::types::signal::Signal;
pub use rhdl_core::types::signal::signal;
pub use rhdl_core::types::timed::Timed;
pub use rhdl_core::types::timed_sample::TimedSample;
pub use rhdl_core::types::timed_sample::timed_sample;
//pub use rhdl_core::{types::bit_string::BitString, util::hash_id};
pub use rhdl_macro::Circuit;
pub use rhdl_macro::CircuitDQ;
pub use rhdl_macro::Digital;
pub use rhdl_macro::Synchronous;
pub use rhdl_macro::SynchronousDQ;
pub use rhdl_macro::Timed;
pub use rhdl_macro::kernel;
pub use rhdl_trace_type as rtt;
// Use the extension traits
pub use rhdl_bits::W;
pub use rhdl_bits::xadd::XAdd;
pub use rhdl_bits::xmul::XMul;
pub use rhdl_bits::xneg::XNeg;
pub use rhdl_bits::xsgn::XSgn;
pub use rhdl_bits::xsub::XSub;
pub use rhdl_core::BitX;
pub use rhdl_core::bitx::bitx_parse;
pub use rhdl_core::bitx::bitx_string;
pub use rhdl_core::bitx_vec;
pub use rhdl_core::circuit::drc;
pub use rhdl_core::circuit::fixture::Driver;
pub use rhdl_core::circuit::fixture::ExportError;
pub use rhdl_core::circuit::fixture::Fixture;
pub use rhdl_core::circuit::fixture::MountPoint;
pub use rhdl_core::circuit::fixture::passthrough_input_driver;
pub use rhdl_core::circuit::fixture::passthrough_output_driver;
pub use rhdl_core::const_max;
pub use rhdl_core::ntl::builder::circuit_black_box;
pub use rhdl_core::ntl::builder::constant;
pub use rhdl_core::ntl::builder::synchronous_black_box;
pub use rhdl_core::sim::extension::*;
pub use rhdl_core::sim::iter::clock_pos_edge::clock_pos_edge;
pub use rhdl_core::sim::iter::merge::merge;
pub use rhdl_core::sim::iter::reset::with_reset;
pub use rhdl_core::sim::iter::reset::without_reset;
pub use rhdl_core::sim::iter::uniform::uniform;
pub use rhdl_core::sim::probe::ext::ProbeExt;
pub use rhdl_core::sim::probe::ext::SynchronousProbeExt;
pub use rhdl_core::sim::run::async_fn::run_async_red_blue;
pub use rhdl_core::sim::run::asynchronous::RunExt;
pub use rhdl_core::sim::run::sync_fn::RunSynchronousFeedbackExt;
pub use rhdl_core::sim::run::synchronous::RunSynchronousExt;
pub use rhdl_core::sim::run::synchronous::RunWithoutSynthesisSynchronousExt;
pub use rhdl_core::sim::testbench::TestBenchOptions;
pub use rhdl_core::sim::testbench::asynchronous::TestBench;
pub use rhdl_core::sim::testbench::synchronous::SynchronousTestBench;
pub use rhdl_core::sim::vcd::Vcd;
pub use rhdl_core::trace::svg::SvgOptions;
pub use rhdl_core::types::path::sub_trace_type;
pub use rhdl_macro::export;
pub use rhdl_macro::path;
pub use rhdl_vlog as vlog;
pub use rhdl_vlog::formatter::Pretty;
pub use rhdl_vlog::parse_quote_miette;

/// A helper macro to bind a named input or output port to a path on the circuit.
///
/// The syntax is either:
/// `bind!(fixture, port_name -> input.<path.to.signal>)`
/// or
/// `bind!(fixture, port_name -> output.<path.to.signal>)`
///
///# Example
///
/// Here is a minimal example of using the `bind!` macro.
///
///```rust
///use rhdl::prelude::*;
///
///#[kernel]
///fn adder(a: Signal<(b4, b4), Red>) -> Signal<b4, Red> {
///    let (a, b) = a.val();
///    signal(a + b) // Return signal with value
///}
///
///let adder = AsyncFunc::new::<adder>()?;
///let mut fixture = Fixture::new("adder_top", adder);
///bind!(fixture, a -> input.val().0);
///bind!(fixture, b -> input.val().1);
///bind!(fixture, sum -> output.val());
///let vlog = fixture.module()?;  
///```
/// When exported as Verilog, the fixture will look like this:
///
///```verilog
///module adder_top(input wire [3:0] a, input wire [3:0] b, output wire [3:0] sum);
///   wire [7:0] inner_input;
///   wire [3:0] inner_output;
///   assign inner_input[3:0] = a;
///   assign inner_input[7:4] = b;
///   assign sum = inner_output[3:0];
///   inner inner_inst(.i(inner_input), .o(inner_output));
///endmodule
///module inner(input wire [7:0] i, output wire [3:0] o);
///   assign o = kernel_adder(i);
///   function [3:0] kernel_adder(input reg [7:0] arg_0);
///         reg [7:0] r0;
///         reg [3:0] r1;
///         reg [3:0] r2;
///         reg [3:0] r3;
///         begin
///            r0 = arg_0;
///            r1 = r0[3:0];
///            r2 = r0[7:4];
///            r3 = r1 + r2;
///            kernel_adder = r3;
///         end
///   endfunction
///endmodule
/// ```
#[macro_export]
macro_rules! bind {
    ($fixture:expr, $name:ident -> input $($path:tt)*) => {
        $fixture.pass_through_input(stringify!($name), &path!($($path)*))?
    };
    ($fixture:expr, $name:ident -> output $($path:tt)*) => {
        $fixture.pass_through_output(stringify!($name), &path!($($path)*))?
    };
}

pub use bind;
