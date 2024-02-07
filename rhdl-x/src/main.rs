use std::iter::repeat;
use std::time::Instant;

use circuit::Circuit;
use counter::Counter;
use counter::CounterI;
use dff::DFF;
use dff::DFFI;
use rhdl_bits::alias::*;
use rhdl_bits::bits;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::note_db::note_time;
use rhdl_core::note_init_db;
use rhdl_core::note_take;
use rhdl_core::DigitalFn;
use rhdl_core::{note, Digital};
use rhdl_macro::kernel;
use rhdl_macro::Digital;
use rhdl_x::Foo;

use rhdl_bits::Bits;
use strobe::Strobe;
use strobe::StrobeI;
use verilog::VerilogTranslator;

mod circuit;
mod clock;
mod constant;
mod counter;
mod dff;
mod strobe;
mod translator;
mod verilog;

// First a DFF

#[test]
fn test_dff() {
    let clock = clock::clock();
    let data = (0..10).cycle();
    let inputs = clock.zip(data).map(|(clock, data)| DFFI { clock, data });
    note_init_db();
    note_time(0);
    let dff = DFF::<u8>::default();
    let mut state = dff.init_state();
    for (time, input) in inputs.enumerate().take(1000) {
        note_time(time as u64 * 1_000);
        note("input", input);
        let output = dff.sim(input, &mut state);
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
    for (time, input) in inputs.enumerate().take(2000) {
        note_time(time as u64 * 100);
        note("input", input);
        let output = strobe.sim(input, &mut state);
        note("output", output);
    }
    let db = note_take().unwrap();
    let strobe = std::fs::File::create("strobe.vcd").unwrap();
    db.dump_vcd(&[], strobe).unwrap();
}

#[test]
fn test_strobe_verilog() {
    let strobe = Strobe::<8>::new(b8(5));
    let top = strobe
        .clone()
        .translate(VerilogTranslator)
        .collect::<anyhow::Result<Vec<_>>>()
        .unwrap()
        .join("\n");
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
    for (time, input) in inputs.enumerate().take(2000) {
        note_time(time as u64 * 100);
        note("input", input);
        let output = counter.sim(input, &mut state);
        note("output", output);
    }
    let db = note_take().unwrap();
    let dff = std::fs::File::create("counter.vcd").unwrap();
    db.dump_vcd(&[], dff).unwrap();
}
