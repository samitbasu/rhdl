use clock_reset::ClockReset;
use rhdl::core::types::timed;
use rhdl::prelude::*;
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
mod clock_reset;
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
                    clock_reset::clock_reset(sample.value, reset(true)),
                    sample.time,
                ))
            } else {
                Some(timed_sample(
                    clock_reset::clock_reset(sample.value, reset(false)),
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
        let output = uut.sim(cr.value.clock, cr.value.reset, input, &mut state, &mut io);
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
                    clock: signal(x.value.clock),
                    reset: signal(x.value.reset),
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

/* #[test]
fn test_counter() {
    let sim_clock = sim_clock(100);
    let sim_cr = sim_clock_reset(sim_clock);
    let inputs = sim_cr
        .map(|x| {
            timed_sample(
                signal(counter::I {
                    cr: x.value,
                    enable: x.time >= 1000 && x.time <= 10000,
                }),
                x.time,
            )
        })
        .take(1000);
    let uut: counter::U<Red, 4> = counter::U::new();
    traced_simulation(uut, inputs, "counter.vcd");
}
 */
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
