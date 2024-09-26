use crate::types::digital::Digital;
use crate::{Circuit, ClockReset, Reset, Synchronous, TimedSample};

type ValidationError = Box<dyn std::error::Error>;
type ValidationResult = Result<(), ValidationError>;

pub trait Validation<C: Circuit> {
    fn initialize(&self, c: &C) -> ValidationResult;
    fn validate(&self, input: TimedSample<C::I>, output: C::O) -> ValidationResult;
    fn finish(&self) -> ValidationResult;
}

pub fn validate<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
    validators: &[Box<dyn Validation<T>>],
) -> ValidationResult {
    for validator in validators {
        validator.initialize(uut)?;
    }
    let mut state = T::S::init();
    let mut io = T::Z::default();
    for sample in inputs {
        let output = uut.sim(sample.value, &mut state, &mut io);
        for validator in validators {
            validator.validate(sample, output)?;
        }
    }
    for validator in validators {
        validator.finish()?;
    }
    Ok(())
}

pub trait SynchronousValidation<S: Synchronous> {
    fn initialize(&self, c: &S) -> ValidationResult;
    fn validate(&self, input: TimedSample<(ClockReset, S::I)>, output: S::O) -> ValidationResult;
    fn finish(&self) -> ValidationResult;
}

pub fn validate_synchronous<T: Synchronous>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<(ClockReset, T::I)>>,
    validators: &[Box<dyn SynchronousValidation<T>>],
) -> ValidationResult {
    for validator in validators {
        validator.initialize(uut)?;
    }
    let mut state = T::S::init();
    let mut io = T::Z::default();
    for timed_input in inputs {
        let clock_reset = timed_input.value.0;
        let input = timed_input.value.1;
        let output = uut.sim(clock_reset, input, &mut state, &mut io);
        for validator in validators {
            validator.validate(timed_input, output)?;
        }
    }
    for validator in validators {
        validator.finish()?;
    }
    Ok(())
}

pub trait PosEdgeValidation<S: Synchronous> {
    fn initialize(&self, c: &S) -> ValidationResult;
    fn validate(&self, reset: Reset, input: S::I, output: S::O) -> ValidationResult;
    fn finish(&self) -> ValidationResult;
}
