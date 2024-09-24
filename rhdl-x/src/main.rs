use rhdl::core::flow_graph::passes::check_for_unconnected_clock_reset::CheckForUnconnectedClockReset;
use rhdl::core::flow_graph::passes::pass::Pass;
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

pub struct TimedSample<T: Digital> {
    pub value: T,
    pub time: u64,
}

pub fn timed_sample<T: Digital>(value: T, time: u64) -> TimedSample<T> {
    TimedSample { value, time }
}

pub fn sim_clock(period: u64) -> impl Iterator<Item = TimedSample<Clock>> {
    (0..).map(move |phase| TimedSample {
        value: clock(phase % 2 == 0),
        time: phase * period,
    })
}

pub fn sim_clock_reset(
    mut clock: impl Iterator<Item = TimedSample<Clock>>,
) -> impl Iterator<Item = TimedSample<ClockReset>> {
    let mut clock_count = 0;
    iter::from_fn(move || {
        if let Some(sample) = clock.next() {
            clock_count += 1;
            if clock_count < 4 {
                Some(timed_sample(
                    clock_reset(sample.value, reset(true)),
                    sample.time,
                ))
            } else {
                Some(timed_sample(
                    clock_reset(sample.value, reset(false)),
                    sample.time,
                ))
            }
        } else {
            None
        }
    })
}

pub fn traced_simulation<T: Circuit>(
    uut: T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
    vcd_filename: &str,
) {
    note_init_db();
    note_time(0);
    let mut state = <T as Circuit>::S::init();
    let mut io = <T as Circuit>::Z::default();
    for sample in inputs {
        note_time(sample.time);
        note("input", sample.value);
        let output = uut.sim(sample.value, &mut state, &mut io);
        note("output", output);
    }
    let db = note_take().unwrap();
    let strobe = std::fs::File::create(vcd_filename).unwrap();
    db.dump_vcd(&[], strobe).unwrap();
}

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

pub fn write_synchronous_testbench<S: Synchronous>(
    uut: S,
    mut inputs: impl Iterator<Item = S::I>,
    v_filename: &str,
) -> Result<(), RHDLError> {
    let out_bits = S::O::bits();
    let in_bits = S::I::bits();
    let in_decl = if in_bits != 0 {
        Some(format!(
            "reg [{in_msb}:0] test_input",
            in_msb = in_bits.saturating_sub(1)
        ))
    } else {
        None
    };
    let out_decl = format!(
        "wire [{out_msb}:0] test_output",
        out_msb = out_bits.saturating_sub(1)
    );
    let file = std::fs::File::create(v_filename).unwrap();
    let mut writer = io::BufWriter::new(file);
    writeln!(writer, "module top;").unwrap();
    writeln!(writer, "reg clock;").unwrap();
    writeln!(writer, "reg reset;").unwrap();
    if let Some(decl) = in_decl {
        writeln!(writer, "{};", decl).unwrap();
    }
    writeln!(writer, "{};", out_decl).unwrap();
    // Add a periodic clock.
    writeln!(writer, "initial begin").unwrap();
    writeln!(writer, "   clock = 1;").unwrap();
    writeln!(writer, "   forever #100 clock = ~clock;").unwrap();
    writeln!(writer, "end").unwrap();
    writeln!(writer, "initial begin").unwrap();
    let clock_stream = sim_clock(100);
    let reset_stream = sim_clock_reset(clock_stream);
    let mut input = S::I::init();
    let mut prev_time = 0_u64;
    let mut reset_prev = false;
    let mut input_prev = S::I::init();
    let hdl = uut.as_hdl(HDLKind::Verilog)?;
    for cr in reset_stream {
        if cr.value.clock.raw() && !cr.value.reset.raw() {
            if let Some(sample) = inputs.next() {
                input = sample;
            } else {
                break;
            }
        }
        let time = cr.time;
        if cr.value.reset.raw() != reset_prev || prev_time == 0 {
            if time != prev_time {
                writeln!(writer, "#{};", time - prev_time).unwrap();
                prev_time = time;
            }
            writeln!(
                writer,
                "reset = {};",
                if cr.value.reset.raw() { 1 } else { 0 }
            )
            .unwrap();
            reset_prev = cr.value.reset.raw();
        }
        if in_bits != 0 && input != input_prev || prev_time == 0 {
            if time != prev_time {
                writeln!(writer, "#{};", time - prev_time).unwrap();
                prev_time = time;
            }
            writeln!(
                writer,
                "test_input = {};",
                input.typed_bits().as_verilog_literal()
            )
            .unwrap();
            input_prev = input;
        }
    }
    writeln!(writer, "end").unwrap();
    if in_bits != 0 {
        writeln!(
            writer,
            "{} dut(.clock(clock), .reset(reset), .i(test_input), .o(test_output));",
            hdl.name
        )
        .unwrap();
    } else {
        writeln!(
            writer,
            "{} dut(.clock(clock), .reset(reset), .o(test_output));",
            hdl.name
        )
        .unwrap();
    }
    writeln!(writer, "initial begin").unwrap();
    writeln!(
        writer,
        "$dumpfile(\"{}.vcd\");",
        v_filename.replace(".", "_")
    )
    .unwrap();
    writeln!(writer, "$dumpvars(0);").unwrap();
    writeln!(writer, "#{};", prev_time).unwrap();
    writeln!(writer, "$finish;").unwrap();
    writeln!(writer, "end").unwrap();
    writeln!(writer, "endmodule").unwrap();
    writeln!(writer, "{:?}", hdl).unwrap();
    Ok(())
}

pub fn traced_synchronous_simulation<S: Synchronous>(
    uut: S,
    mut inputs: impl Iterator<Item = S::I>,
    vcd_filename: &str,
) {
    note_init_db();
    note_time(0);
    let clock_stream = sim_clock(100);
    let reset_stream = sim_clock_reset(clock_stream);
    let mut state = S::S::init();
    let mut input = S::I::init();
    let mut io = S::Z::default();
    for cr in reset_stream {
        if cr.value.clock.raw() && !cr.value.reset.raw() {
            if let Some(sample) = inputs.next() {
                input = sample;
            } else {
                break;
            }
        }
        note_time(cr.time);
        note("clock", cr.value.clock);
        note("reset", cr.value.reset);
        note("input", input);
        let output = uut.sim(cr.value, input, &mut state, &mut io);
        if S::Z::N != 0 {
            note("io", io);
        }
        note("output", output);
    }
    let db = note_take().unwrap();
    let strobe = std::fs::File::create(vcd_filename).unwrap();
    db.dump_vcd(&[], strobe).unwrap();
}

#[test]
fn test_async_counter() {
    let clock_stream = sim_clock(100);
    let reset_stream = sim_clock_reset(clock_stream);
    let inputs = reset_stream
        .map(|x| {
            timed_sample(
                async_counter::I {
                    clock_reset: signal(x.value),
                    enable: signal(counter::I {
                        enable: x.time >= 1000 && x.time <= 10000,
                    }),
                },
                x.time,
            )
        })
        .take_while(|x| x.time < 20000);
    let uut: async_counter::U = async_counter::U::default();
    traced_simulation(uut, inputs, "async_counter.vcd")
}

#[test]
fn test_async_counter_fg() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let fg = uut.descriptor()?.flow_graph.sealed();
    let fg = CheckForUnconnectedClockReset::run(fg)?;
    let mut dot = std::fs::File::create("async_counter.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    let hdl = uut.as_hdl(HDLKind::Verilog)?;
    eprintln!("{:?}", hdl);
    Ok(())
}

#[test]
fn test_async_counter_hdl() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let hdl = uut.as_hdl(HDLKind::Verilog)?;
    eprintln!("{:?}", hdl);
    Ok(())
}

#[test]
fn test_adapter_fg() -> miette::Result<()> {
    let counter = counter::U::new();
    let uut = Adapter::<counter::U<2>, Red>::new(counter);
    let fg = &uut.descriptor()?.flow_graph.sealed();
    let mut dot = std::fs::File::create("adapter.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    Ok(())
}

#[test]
fn test_dff() {
    let inputs = (0..).map(|_| Bits::init()).take(1000);
    let uut: dff::U<b4> = dff::U::new(b4::from(0b0000));
    traced_synchronous_simulation(uut, inputs, "dff.vcd");
}

#[test]
fn test_constant() {
    let inputs = (0..).map(|_| ()).take(100);
    let uut: constant::U<b4> = constant::U::new(b4::from(0b1010));
    traced_synchronous_simulation(uut, inputs, "constant.vcd");
}

#[test]
fn test_strobe() {
    let inputs = (0..).map(|_| strobe::I { enable: true }).take(1000);
    let uut: strobe::U<16> = strobe::U::new(bits(100));
    traced_synchronous_simulation(uut, inputs, "strobe.vcd");
}

#[test]
fn test_strobe_fg() -> miette::Result<()> {
    let uut: strobe::U<8> = strobe::U::new(bits(100));
    let fg = &uut.descriptor()?.flow_graph.sealed();
    let mut dot = std::fs::File::create("strobe.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    Ok(())
}

#[test]
fn test_counter_simulation() {
    let inputs = (0..5000)
        .map(|x| x > 1000 && x < 10000)
        .map(|x| counter::I { enable: x });
    let uut: counter::U<4> = counter::U::new();
    traced_synchronous_simulation(uut, inputs, "counter.vcd");
}

#[test]
fn test_counter_testbench() -> miette::Result<()> {
    let inputs = (0..1000)
        .map(|x| x > 100 && x < 900)
        .map(|x| counter::I { enable: x });
    let uut: counter::U<4> = counter::U::new();
    write_synchronous_testbench(uut, inputs, "counter_tb.v")?;
    Ok(())
}

fn main() -> miette::Result<()> {
    let counter: counter::U<4> = counter::U::new();
    let hdl = counter.as_hdl(HDLKind::Verilog)?;
    println!("{}", hdl.body);
    for (child, descriptor) in hdl.children {
        println!("{child} {}", descriptor.body);
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
