use rhdl::core::flow_graph::passes::check_for_unconnected_clock_reset::CheckForUnconnectedClockReset;
use rhdl::core::flow_graph::passes::pass::Pass;
use rhdl::core::flow_graph::verilog::generate_verilog;
use rhdl::core::types::timed;
use rhdl::prelude::*;
use std::io::Write;
use std::iter::repeat;
use std::{io, iter};

use anyhow::ensure;
/* use rhdl_core::as_verilog_literal;
use rhdl_core::codegen::verilog::as_verilog_decl;
use rhdl_core::prelude::*;
use rhdl_core::root_descriptor;
use rhdl_core::types::domain::Red;
use rhdl_macro::Digital;
use rhdl_macro::Timed;
 */
//use translator::Translator;
//use verilog::VerilogTranslator;

//mod backend;
//mod circuit;
//mod clock;
mod auto_counter;
mod constant;
mod counter;
mod strobe;
//mod descriptions;
mod dff;
pub mod inverter;
pub mod logic_loop;
pub mod single_bit;
//mod push_pull;
//mod strobe;
//mod tristate;
//mod traitx;
//mod translator;
//mod verilog;
//mod dfg;
//mod trace;
//mod case;
//mod check;
//mod signal;
//mod timeset;
//mod visit;
mod async_counter;

//#[cfg(test)]
//mod tests;

// Let's start with the DFF.  For now, we will assume a reset is present.

// Template:
/*
    module top;
    reg clk;
    reg enable;
    wire out;

    initial begin
       enable = 1;
       clk = 1;
       forever #10 clk = ~clk;
    end

    Strobe_748a98de03e4aa30 dut(.i({enable, clk}), .o(out) );

    initial begin
    $dumpfile("strobe_v.vcd");
    $dumpvars(0);
    #1000;
    $finish;
    end

  endmodule

*/

#[test]
fn test_async_counter() {
    let inputs = (0..1000)
        .map(|x| x > 100 && x < 900)
        .map(|x| counter::I { enable: x });
    let inputs = test_stream(inputs);
    let inputs = inputs.map(|x| {
        timed_sample(
            async_counter::I {
                clock_reset: signal(x.value.0),
                enable: signal(x.value.1),
            },
            x.time,
        )
    });
    let uut: async_counter::U = async_counter::U::default();
    traced_simulation(&uut, inputs, "async_counter.vcd")
}

#[test]
fn test_async_counter_fg() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let hdl = uut.hdl()?;
    eprintln!("{}", hdl.as_verilog());
    let fg = uut.flow_graph()?;
    let mut dot = std::fs::File::create("async_counter.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    Ok(())
}

// TO check with yosys:
// yosys -p "read -vlog95 async_counter.v; hierarchy -check -top rhdl_x_async_counter_U_fb5e6b876dbb9038; proc; write -vlog95 async_counter_yosys.v"
#[test]
fn test_async_counter_hdl() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let hdl = uut.hdl()?;
    eprintln!("{}", hdl.as_verilog());
    std::fs::write("async_counter.v", hdl.as_verilog()).unwrap();
    Ok(())
}

#[test]
fn test_async_counter_tb() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let inputs = (0..1000)
        .map(|x| x > 100 && x < 900)
        .map(|x| counter::I { enable: x });
    let inputs = test_stream(inputs);
    let inputs = inputs.map(|x| {
        timed_sample(
            async_counter::I {
                clock_reset: signal(x.value.0),
                enable: signal(x.value.1),
            },
            x.time,
        )
    });
    write_testbench(&uut, inputs, "async_counter_tb.v")?;
    Ok(())
}

#[test]
fn test_adapter_fg() -> miette::Result<()> {
    let counter = counter::U::default();
    let uut = Adapter::<counter::U<2>, Red>::new(counter);
    let fg = &uut.descriptor()?.flow_graph.sealed();
    let mut dot = std::fs::File::create("adapter.dot").unwrap();
    write_dot(fg, &mut dot).unwrap();
    Ok(())
}

#[cfg(test)]
fn test_stream<T: Digital>(
    inputs: impl Iterator<Item = T>,
) -> impl Iterator<Item = TimedSample<(ClockReset, T)>> {
    stream::clock_pos_edge(stream::reset_pulse(4).chain(stream::stream(inputs)), 100)
}

#[test]
fn test_dff() {
    let inputs = (0..).map(|_| Bits::init()).take(1000);
    let uut: dff::U<b4> = dff::U::new(b4::from(0b0000));
    traced_synchronous_simulation(&uut, test_stream(inputs), "dff.vcd");
}

#[test]
fn test_constant() {
    let inputs = (0..).map(|_| ()).take(100);
    let uut: constant::U<b4> = constant::U::new(b4::from(0b1010));
    traced_synchronous_simulation(&uut, test_stream(inputs), "constant.vcd");
}

#[test]
fn test_strobe() {
    let inputs = (0..).map(|_| strobe::I { enable: true }).take(1000);
    let uut: strobe::U<16> = strobe::U::new(bits(100));
    traced_synchronous_simulation(&uut, test_stream(inputs), "strobe.vcd");
}

#[test]
fn test_strobe_fg() -> miette::Result<()> {
    let uut: strobe::U<8> = strobe::U::new(bits(100));
    let fg = &uut.flow_graph()?.sealed();
    let mut dot = std::fs::File::create("strobe.dot").unwrap();
    write_dot(fg, &mut dot).unwrap();
    Ok(())
}

#[test]
fn test_counter_simulation() {
    let inputs = (0..5000)
        .map(|x| x > 1000 && x < 10000)
        .map(|x| counter::I { enable: x });
    let uut: counter::U<4> = counter::U::default();
    traced_synchronous_simulation(&uut, test_stream(inputs), "counter.vcd");
}

#[test]
fn test_counter_testbench() -> miette::Result<()> {
    let inputs = (0..1000)
        .map(|x| x > 100 && x < 900)
        .map(|x| counter::I { enable: x });
    let inputs = stream::reset_pulse(1).chain(stream::stream(inputs));
    let uut: counter::U<4> = counter::U::default();
    write_synchronous_testbench(&uut, inputs, 100, "counter_tb.v")?;
    Ok(())
}

#[test]
fn test_autocounter() -> miette::Result<()> {
    let uut: auto_counter::U<4> = auto_counter::U::default();
    let fg = uut.flow_graph()?;
    let vg = generate_verilog("top", &fg)?;
    std::fs::write(
        "auto_counter.v",
        rhdl::core::hdl::formatter::function(&vg.functions[0]),
    )
    .unwrap();
    let mut dot = std::fs::File::create("auto_counter.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    Ok(())
}

fn main() -> miette::Result<()> {
    let counter: counter::U<4> = counter::U::default();
    let hdl = counter.hdl()?;
    println!("{}", hdl.as_verilog());
    for (child, descriptor) in hdl.children {
        println!("{child} {}", descriptor.as_verilog());
    }
    /*
       let strobe: strobe::U<16> = strobe::U::new(bits(100));
       let hdl = strobe.as_hdl(HDLKind::Verilog)?;
       println!("{}", hdl.body);

       let dff: dff::U<b4> = dff::U::new(b4::from(0b1010));
       let hdl = dff.as_hdl(HDLKind::Verilog)?;
       println!("{}", hdl.body);
    */
    Ok(())
}
