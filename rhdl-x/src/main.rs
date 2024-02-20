use std::iter::repeat;
use std::time::Instant;

use circuit::BufZ;
use circuit::Circuit;
use circuit::TristateBuf;
use counter::Counter;
use counter::CounterI;
use dff::DFF;
use dff::DFFI;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use rhdl_bits::alias::*;
use rhdl_bits::bits;
use rhdl_core::compile_design;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::note_db::note_time;
use rhdl_core::note_init_db;
use rhdl_core::note_pop_path;
use rhdl_core::note_push_path;
use rhdl_core::note_take;
use rhdl_core::DigitalFn;
use rhdl_core::{note, Digital};
use rhdl_macro::kernel;
use rhdl_macro::Digital;

use rhdl_bits::Bits;
use strobe::Strobe;
use strobe::StrobeI;

use crate::dfg::build_dfg;
use crate::dfg::ObjectAnalyzer;
//use translator::Translator;
//use verilog::VerilogTranslator;

use std::fmt::Write;
//mod backend;
mod circuit;
mod clock;
mod constant;
mod counter;
mod descriptions;
mod dff;
mod push_pull;
mod strobe;
mod tristate;
//mod traitx;
//mod translator;
//mod verilog;
mod dfg;
mod visit;

#[test]
fn test_dff() {
    let clock = clock::clock();
    let data = (0..10).cycle();
    let inputs = clock.zip(data).map(|(clock, data)| DFFI { clock, data });
    note_init_db();
    note_time(0);
    let dff = DFF::<u8>::default();
    let mut state = dff.init_state();
    let mut io = <DFF<u8> as Circuit>::Z::default();
    for (time, input) in inputs.enumerate().take(1000) {
        note_time(time as u64 * 1_000);
        note("input", input);
        let output = dff.sim(input, &mut state, &mut io);
        note("output", output);
    }
    let db = note_take().unwrap();
    let dff = std::fs::File::create("dff.vcd").unwrap();
    db.dump_vcd(&[], dff).unwrap();
}

#[test]
fn test_strobe() {
    let clock = clock::clock();
    let enable = std::iter::repeat(true);
    let inputs = clock
        .zip(enable)
        .map(|(clock, enable)| StrobeI { clock, enable });
    note_init_db();
    note_time(0);
    let strobe = Strobe::<8>::new(b8(5));
    let mut state = strobe.init_state();
    let mut io = <Strobe<8> as Circuit>::Z::default();
    for (time, input) in inputs.enumerate().take(2000) {
        note_time(time as u64 * 100);
        note("input", input);
        let output = strobe.sim(input, &mut state, &mut io);
        note("output", output);
    }
    let db = note_take().unwrap();
    let strobe = std::fs::File::create("strobe.vcd").unwrap();
    db.dump_vcd(&[], strobe).unwrap();
}

#[test]
fn test_strobe_verilog() {
    let strobe = Strobe::<8>::new(b8(5));
    let top = strobe.as_hdl(crate::circuit::HDLKind::Verilog).unwrap();
    let verilog = format!(
        "
    module top;
    reg clk;
    reg enable;
    wire out;
  
    initial begin
       enable = 1;
       clk = 1;
       forever #10 clk = ~clk;
    end
  
    Strobe_748a98de03e4aa30 dut(.i({{enable, clk}}), .o(out) );
  
    initial begin
    $dumpfile(\"strobe_v.vcd\");
    $dumpvars(0);
    #1000;
    $finish;
    end
  
  
  endmodule
    
    {}",
        top
    );
    std::fs::write("strobe.v", verilog).unwrap();
}

fn main() {
    let clock = clock::clock();
    let enable = std::iter::repeat(false)
        .take(20)
        .chain(std::iter::repeat(true));
    let inputs = clock
        .zip(enable)
        .map(|(clock, enable)| CounterI { clock, enable });
    note_init_db();
    note_time(0);
    let counter = Counter::<8>::default();
    let mut state = counter.init_state();
    let mut io = <Counter<8> as Circuit>::Z::default();
    for (time, input) in inputs.enumerate().take(2000) {
        note_time(time as u64 * 100);
        note("input", input);
        let output = counter.sim(input, &mut state, &mut io);
        note("output", output);
    }
    let db = note_take().unwrap();
    let dff = std::fs::File::create("counter.vcd").unwrap();
    db.dump_vcd(&[], dff).unwrap();
}

#[test]
fn test_timing_note() {
    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum State {
        #[default]
        A,
        B,
        C,
    };
    note_init_db();
    note_time(0);
    let tic = Instant::now();
    for i in 0..1_000_000 {
        note_time(i);
        note_push_path("a");
        note_push_path("b");
        note_push_path("c");
        note("i", b8(4));
        note_pop_path();
        note_pop_path();
        note_pop_path();
        note_push_path("a");
        note_push_path("b");
        note_push_path("d");
        note("name", b16(0x1234));
        note_pop_path();
        note_pop_path();
        note_pop_path();
        note_push_path("b");
        note_push_path("c");
        note_push_path("e");
        note("color", State::B);
        note_pop_path();
        note_pop_path();
        note_pop_path();
    }
    let toc = Instant::now();
    eprintln!("Time: {:?}", toc - tic);
}

#[test]
fn test_dfg_analysis_of_kernel() {
    #[kernel]
    fn concatenate_bits(x: b4, y: b4) -> b4 {
        y - x
    }

    #[kernel]
    fn add_stuff(x: b4, y: b4) -> b4 {
        x + concatenate_bits(x, y)
    }

    let design = compile_design(add_stuff::kernel_fn().unwrap().try_into().unwrap()).unwrap();
    let mut dfg = build_dfg(&design, design.top).unwrap();
    eprintln!("{:?}", dfg);
    // Print out the DFG graph as a DOT file
    let mut dot = String::new();
    writeln!(
        dot,
        "{}",
        Dot::with_config(&dfg.graph, &[Config::EdgeNoLabel])
    )
    .unwrap();
    std::fs::write("dfg.dot", dot).unwrap();
}
