use super::{sim_clock, sim_clock_reset, TimedSample};
use crate::types::digital::Digital;
use crate::types::tristate::Tristate;
use crate::{note, note_init_db, note_take, note_time, Circuit, Synchronous};

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
