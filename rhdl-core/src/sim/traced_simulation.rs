use crate::types::digital::Digital;
use crate::{
    note, note_init_db, note_time, Circuit, CircuitDQZ, ClockReset, Synchronous, TimedSample,
};

pub fn traced_simulation<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
    vcd_filename: &str,
) {
    let guard = note_init_db();
    note_time(0);
    let mut state = <T as Circuit>::S::init();
    let mut io = <T as CircuitDQZ>::Z::default();
    for sample in inputs {
        note_time(sample.time);
        note("input", sample.value);
        let output = uut.sim(sample.value, &mut state, &mut io);
        note("output", output);
    }
    let db = guard.take();
    let strobe = std::fs::File::create(vcd_filename).unwrap();
    db.dump_vcd(strobe).unwrap();
}

pub fn traced_synchronous_simulation<S: Synchronous>(
    uut: &S,
    inputs: impl Iterator<Item = TimedSample<(ClockReset, S::I)>>,
    vcd_filename: &str,
) {
    let guard = note_init_db();
    note_time(0);
    let mut state = S::S::init();
    let mut io = S::Z::default();
    for timed_input in inputs {
        note_time(timed_input.time);
        let clock_reset = timed_input.value.0;
        let input = timed_input.value.1;
        note("clock", clock_reset.clock);
        note("reset", clock_reset.reset);
        note("input", input);
        let output = uut.sim(clock_reset, input, &mut state, &mut io);
        note("io", io);
        note("output", output);
    }
    let db = guard.take();
    let strobe = std::fs::File::create(vcd_filename).unwrap();
    db.dump_vcd(strobe).unwrap();
}
