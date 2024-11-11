use crate::{
    sim::validation_simulation::SynchronousValidation, Clock, ClockReset, Digital, Synchronous,
    TimedSample,
};

#[derive(Debug, Default)]
struct GlitchCheck<T> {
    clk: Clock,
    prev_val: T,
    initialized: bool,
}

impl<S> SynchronousValidation<S> for GlitchCheck<S::O>
where
    S: Synchronous,
{
    fn validate(&mut self, input: TimedSample<(ClockReset, S::I)>, output: S::O) {
        let clock = input.value.0.clock;
        if self.initialized {
            let pos_edge = clock.raw() && !self.clk.raw();
            let output_changed = output != self.prev_val;
            if output_changed && !pos_edge {
                panic!("Glitch detected at time: {time}", time = input.time);
            }
        }
        self.initialized = true;
        self.clk = clock;
        self.prev_val = output;
    }
}

pub fn glitch_check_synchronous<S: Synchronous>() -> Box<dyn SynchronousValidation<S>> {
    Box::new(GlitchCheck {
        clk: Clock::init(),
        prev_val: S::O::init(),
        initialized: false,
    })
}
