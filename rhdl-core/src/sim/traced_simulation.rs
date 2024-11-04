use crate::types::digital::Digital;
use crate::{
    trace, trace_init_db, trace_time, Circuit, CircuitDQZ, ClockReset, Synchronous, TimedSample,
};

pub fn traced_simulation<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
    vcd_filename: &str,
) {
    let guard = trace_init_db();
    trace_time(0);
    let mut state = <T as Circuit>::S::init();
    let mut io = <T as CircuitDQZ>::Z::default();
    for sample in inputs {
        trace_time(sample.time);
        trace("input", &sample.value);
        let output = uut.sim(sample.value, &mut state, &mut io);
        trace("output", &output);
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
    let guard = trace_init_db();
    trace_time(0);
    let mut state = S::S::init();
    let mut io = S::Z::default();
    for timed_input in inputs {
        trace_time(timed_input.time);
        let clock_reset = timed_input.value.0;
        let input = timed_input.value.1;
        trace("clock", &clock_reset.clock);
        trace("reset", &clock_reset.reset);
        trace("input", &input);
        let output = uut.sim(clock_reset, input, &mut state, &mut io);
        trace("output", &output);
    }
    let db = guard.take();
    let strobe = std::fs::File::create(vcd_filename).unwrap();
    let strobe = std::io::BufWriter::new(strobe);
    db.dump_vcd(strobe).unwrap();
    let rtt = std::fs::File::create(format!("{}.rtt", vcd_filename)).unwrap();
    let rtt = std::io::BufWriter::new(rtt);
    db.dump_rtt(rtt).unwrap();
}
