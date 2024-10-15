use crate::{Circuit, Clock, ClockReset, Digital, Kind, Reset, Synchronous, TimedSample};

#[derive(Clone, Debug)]
pub struct AsynchronousEntry {
    pub delay: u64,
    pub input: Vec<bool>,
    pub output: Vec<bool>,
    pub io: Vec<bool>,
}

#[derive(Clone, Debug)]
pub struct SynchronousEntry {
    pub delay: u64,
    pub clock: Clock,
    pub reset: Reset,
    pub input: Vec<bool>,
    pub output: Vec<bool>,
    pub io: Vec<bool>,
}

#[derive(Clone, Debug)]
pub struct AsynchronousWaveform {
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub entries: Vec<AsynchronousEntry>,
}

#[derive(Clone, Debug)]
pub struct SynchronousWaveform {
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub entries: Vec<SynchronousEntry>,
}

pub fn waveform<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
) -> AsynchronousWaveform {
    let mut state = <T as Circuit>::S::init();
    let mut io = <T as Circuit>::Z::default();
    let mut previous_time = 0;
    let mut entries = Vec::new();
    for sample in inputs {
        let time = sample.time;
        let input = sample.value;
        let output = uut.sim(input, &mut state, &mut io);
        let entry = AsynchronousEntry {
            delay: time - previous_time,
            input: input.bin(),
            output: output.bin(),
            io: vec![],
        };
        previous_time = time;
        entries.push(entry);
    }
    AsynchronousWaveform {
        input_kind: T::I::static_kind(),
        output_kind: T::O::static_kind(),
        entries,
    }
}

pub fn waveform_synchronous<T: Synchronous>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<(ClockReset, T::I)>>,
) -> SynchronousWaveform {
    let mut state = T::S::init();
    let mut io = T::Z::default();
    let mut previous_time = 0;
    let mut entries = Vec::new();
    for timed_input in inputs {
        let time = timed_input.time;
        let clock_reset = timed_input.value.0;
        let input = timed_input.value.1;
        let output = uut.sim(clock_reset, input, &mut state, &mut io);
        let entry = SynchronousEntry {
            delay: time - previous_time,
            clock: clock_reset.clock,
            reset: clock_reset.reset,
            input: input.bin(),
            output: output.bin(),
            io: vec![],
        };
        previous_time = time;
        entries.push(entry);
    }
    SynchronousWaveform {
        input_kind: T::I::static_kind(),
        output_kind: T::O::static_kind(),
        entries,
    }
}
