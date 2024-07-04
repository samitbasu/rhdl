use rhdl::prelude::*;
use std::iter;
use std::iter::repeat;

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
//mod constant;
//mod counter;
//mod descriptions;
mod dff;
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

pub fn sim_reset(
    mut clock: impl Iterator<Item = TimedSample<Clock>>,
) -> impl Iterator<Item = TimedSample<(Clock, Reset)>> {
    let mut clock_count = 0;
    iter::from_fn(move || {
        if let Some(sample) = clock.next() {
            clock_count += 1;
            if clock_count < 4 {
                Some(timed_sample((sample.value, reset(true)), sample.time))
            } else {
                Some(timed_sample((sample.value, reset(false)), sample.time))
            }
        } else {
            None
        }
    })
}

pub fn sim_samples<T: Digital>(
    period: u64,
) -> impl Iterator<Item = TimedSample<Signal<dff::I<T>, Red>>> {
    let mut input = sim_reset(sim_clock(period));
    let mut sig_value = T::random();
    iter::from_fn(move || {
        if let Some(sample) = input.next() {
            if sample.value.0.raw() {
                sig_value = T::random();
            }
            Some(timed_sample(
                signal(dff::I {
                    data: sig_value,
                    clock: sample.value.0,
                    reset: sample.value.1,
                }),
                sample.time,
            ))
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
    let mut state = <T as Circuit>::S::random();
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

#[test]
fn test_dff() {
    let inputs = sim_samples(1000).take(1000);
    let uut: dff::U<b4, Red> = dff::U::new(b4::from(0b0000));
    traced_simulation(uut, inputs, "dff.vcd");
}

fn main() -> anyhow::Result<()> {
    let dff: dff::U<b4, Red> = dff::U::new(b4::from(0b1010));
    let hdl = dff.as_hdl(HDLKind::Verilog)?;
    println!("{}", hdl.body);
    Ok(())
}
