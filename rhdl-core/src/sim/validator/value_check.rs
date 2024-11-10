use crate::{
    sim::validation_simulation::{Validation, ValidationResult},
    Circuit, CircuitIO, Clock, Digital, TimedSample,
};

#[derive(Debug, Default)]
struct ValueCheck<F, I> {
    clk: Clock,
    func: F,
    initialized: bool,
    expected: I,
}

impl<F, C, I> Validation<C> for ValueCheck<F, I>
where
    C: Circuit,
    F: Fn(&TimedSample<<C as CircuitIO>::I>) -> Clock,
    I: Iterator<Item = Option<<C as CircuitIO>::O>>,
    <C as CircuitIO>::O: std::fmt::Debug,
{
    fn validate(
        &mut self,
        input: TimedSample<<C as CircuitIO>::I>,
        output: <C as CircuitIO>::O,
    ) -> ValidationResult {
        let clock = (self.func)(&input);
        if self.initialized {
            let pos_edge = clock.raw() && !self.clk.raw();
            if pos_edge {
                if let Some(Some(val)) = self.expected.next() {
                    if val != output {
                        panic!(
                            "Expected value {val:?} but got {output:?} at time: {time}",
                            time = input.time
                        );
                    }
                }
            }
        }
        self.initialized = true;
        self.clk = clock;
        Ok(())
    }
}

pub fn value_check<C>(
    func: impl Fn(&TimedSample<C::I>) -> Clock + 'static,
    expected: impl Iterator<Item = Option<C::O>> + 'static,
) -> Box<dyn Validation<C>>
where
    C: Circuit,
    <C as CircuitIO>::O: std::fmt::Debug,
{
    Box::new(ValueCheck {
        clk: Clock::init(),
        func,
        initialized: false,
        expected: expected.fuse(),
    })
}
