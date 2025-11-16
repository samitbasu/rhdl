use crate::sim::extension::*;
use crate::types::clock_reset::clock_reset;
use crate::types::reset::reset;
use crate::types::signal::signal;
use crate::{Circuit, CircuitIO, TimedSample};
use crate::{ClockReset, Digital};
use crate::{Domain, Signal, clock::clock, trace_time};

fn neg_edge<D: Domain>(prev_cr: Signal<ClockReset, D>, curr_cr: Signal<ClockReset, D>) -> bool {
    let prev_cr = prev_cr.val();
    let curr_cr = curr_cr.val();
    if curr_cr.reset.any() || prev_cr.reset.any() {
        return false;
    }
    if !curr_cr.clock.raw() && prev_cr.clock.raw() {
        return true;
    }
    false
}

pub fn run_async_red_blue<'a, T, FR, FB, FJ, R, B>(
    uut: &'a T,
    mut red_fn: FR,
    mut blue_fn: FB,
    red_period: u64,
    blue_period: u64,
    injector: FJ,
) -> impl Iterator<Item = TimedSample<(<T as CircuitIO>::I, <T as CircuitIO>::O)>> + 'a
where
    T: Circuit,
    FJ: Fn(Signal<ClockReset, R>, Signal<ClockReset, B>, &mut <T as CircuitIO>::I) + 'a,
    FR: FnMut(<T as CircuitIO>::O, &mut <T as CircuitIO>::I) + 'a,
    FB: FnMut(<T as CircuitIO>::O, &mut <T as CircuitIO>::I) + 'a,
    R: Domain,
    B: Domain,
{
    let mut prev_output = <T as CircuitIO>::O::dont_care();
    let mut state = uut.init();
    let mut prev_input = <T as CircuitIO>::I::dont_care();
    let mut prev_red_cr = signal::<_, R>(clock_reset(clock(false), reset(true)));
    let mut prev_blue_cr = signal::<_, B>(clock_reset(clock(false), reset(true)));
    let red_input = std::iter::repeat(())
        .with_reset(1)
        .clock_pos_edge(red_period);
    let blue_input = std::iter::repeat(())
        .with_reset(1)
        .clock_pos_edge(blue_period);
    let mut sequence = red_input.merge_map(blue_input, |r, b| (r, b));
    std::iter::from_fn(move || {
        if let Some(event) = sequence.next() {
            trace_time(event.time);
            let red_cr = signal(event.value.0.0);
            let blue_cr = signal(event.value.1.0);
            let mut input = prev_input;
            injector(red_cr, blue_cr, &mut input);
            if neg_edge(prev_red_cr, red_cr) {
                red_fn(prev_output, &mut input);
            }
            if neg_edge(prev_blue_cr, blue_cr) {
                blue_fn(prev_output, &mut input);
            }
            let output = uut.sim(input, &mut state);
            prev_input = input;
            prev_output = output;
            prev_blue_cr = blue_cr;
            prev_red_cr = red_cr;
            Some(event.map(|_| (input, output)))
        } else {
            None
        }
    })
}
