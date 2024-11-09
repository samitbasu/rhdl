use crate::{Circuit, TimedSample};
use crate::{ClockReset, Digital, Synchronous};

pub fn final_output_simulation<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
) -> Option<T::O> {
    let mut state = <T as Circuit>::S::init();
    let mut output = None;
    for sample in inputs {
        output = Some(uut.sim(sample.value, &mut state));
    }
    output
}

pub fn final_output_synchronous_simulation<S: Synchronous>(
    uut: &S,
    inputs: impl Iterator<Item = TimedSample<(ClockReset, S::I)>>,
) -> Option<S::O> {
    let mut state = S::S::init();
    let mut output = None;
    for timed_input in inputs {
        let clock = timed_input.value.0;
        let input = timed_input.value.1;
        output = Some(uut.sim(clock, input, &mut state));
    }
    output
}
