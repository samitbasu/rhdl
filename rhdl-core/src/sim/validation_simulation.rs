use crate::{
    trace::{db::trace, db::with_trace_db},
    trace_init_db, trace_time, Circuit, ClockReset, Synchronous, TimedSample,
};

pub trait Validation<C: Circuit> {
    fn initialize(&mut self, _c: &C) {}
    fn validate(&mut self, _input: TimedSample<C::I>, _output: C::O) {}
    fn finish(&mut self) {}
}

#[derive(Default, Debug, Clone)]
pub struct ValidateOptions {
    pub vcd_filename: Option<String>,
}

impl ValidateOptions {
    fn needs_trace_db(&self) -> bool {
        self.vcd_filename.is_some()
    }
    fn write_files(self) {
        if let Some(vcd_filename) = self.vcd_filename {
            with_trace_db(|db| {
                let strobe = std::fs::File::create(&vcd_filename).unwrap();
                let strobe = std::io::BufWriter::new(strobe);
                db.dump_vcd(strobe).unwrap()
            });
        }
    }
    pub fn vcd(self, name: &str) -> Self {
        Self {
            vcd_filename: Some(name.into()),
        }
    }
}

pub fn simple_traced_run<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
    vcd_filename: &str,
) {
    validate(
        uut,
        inputs,
        &mut [],
        ValidateOptions::default().vcd(vcd_filename),
    );
}

pub fn validate<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
    validators: &mut [Box<dyn Validation<T>>],
    config: ValidateOptions,
) {
    for validator in validators.iter_mut() {
        validator.initialize(uut);
    }
    let _guard = if config.needs_trace_db() {
        Some(trace_init_db())
    } else {
        None
    };
    let mut state = uut.init();
    let mut time = 0;
    for sample in inputs {
        assert!(sample.time >= time);
        time = sample.time;
        trace_time(time);
        trace("input", &sample.value);
        let output = uut.sim(sample.value, &mut state);
        trace("output", &output);
        for validator in validators.iter_mut() {
            validator.validate(sample, output);
        }
    }
    for validator in validators {
        validator.finish();
    }
    config.write_files();
}

pub trait SynchronousValidation<S: Synchronous> {
    fn initialize(&mut self, _c: &S) {}
    fn validate(&mut self, _input: TimedSample<(ClockReset, S::I)>, _output: S::O) {}
    fn finish(&mut self) {}
}

pub fn validate_synchronous<T: Synchronous>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<(ClockReset, T::I)>>,
    validators: &mut [Box<dyn SynchronousValidation<T>>],
    config: ValidateOptions,
) {
    for validator in validators.iter_mut() {
        validator.initialize(uut);
    }
    let _guard = if config.needs_trace_db() {
        Some(trace_init_db())
    } else {
        None
    };
    let mut state = uut.init();
    let mut time = 0;
    for timed_input in inputs {
        assert!(timed_input.time >= time);
        time = timed_input.time;
        trace_time(time);
        let clock_reset = timed_input.value.0;
        let input = timed_input.value.1;
        trace("clock", &clock_reset.clock);
        trace("reset", &clock_reset.reset);
        trace("input", &input);
        let output = uut.sim(clock_reset, input, &mut state);
        trace("output", &output);
        for validator in validators.iter_mut() {
            validator.validate(timed_input, output);
        }
    }
    for validator in validators {
        validator.finish();
    }
    config.write_files();
}

pub fn simple_traced_synchronous_run<T: Synchronous>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<(ClockReset, T::I)>>,
    vcd_filename: &str,
) {
    validate_synchronous(
        uut,
        inputs,
        &mut [],
        ValidateOptions::default().vcd(vcd_filename),
    );
}
