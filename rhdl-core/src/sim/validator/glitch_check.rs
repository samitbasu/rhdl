use crate::{
    sim::validation_simulation::Validation, Circuit, CircuitIO, Clock, Digital, TimedSample,
};

#[derive(Debug, Default)]
struct GlitchCheck<T, F> {
    clk: Clock,
    prev_val: T,
    func: F,
    initialized: bool,
}

impl<F, C> Validation<C> for GlitchCheck<<C as CircuitIO>::O, F>
where
    C: Circuit,
    F: Fn(&TimedSample<<C as CircuitIO>::I>) -> Clock,
{
    fn validate(&mut self, input: TimedSample<<C as CircuitIO>::I>, output: <C as CircuitIO>::O) {
        let clock = (self.func)(&input);
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

pub fn glitch_check<C: Circuit>(
    func: impl Fn(&TimedSample<C::I>) -> Clock + 'static,
) -> Box<dyn Validation<C>> {
    Box::new(GlitchCheck {
        clk: Clock::init(),
        prev_val: C::O::init(),
        func,
        initialized: false,
    })
}
