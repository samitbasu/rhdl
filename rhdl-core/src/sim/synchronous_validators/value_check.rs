use crate::{
    sim::validation_simulation::SynchronousValidation, Clock, ClockReset, Digital, Synchronous,
    TimedSample,
};

#[derive(Debug, Default)]
struct ValueCheck<I> {
    clk: Clock,
    initialized: bool,
    expected: I,
}

impl<S, I> SynchronousValidation<S> for ValueCheck<I>
where
    S: Synchronous,
    I: Iterator<Item = Option<S::O>>,
    S::O: std::fmt::Debug,
{
    fn validate(&mut self, input: TimedSample<(ClockReset, S::I)>, output: S::O) {
        let clock = input.value.0.clock;
        let reset = input.value.0.reset;
        if self.initialized {
            let pos_edge = clock.raw() && !self.clk.raw();
            if pos_edge && !reset.raw() {
                if let Some(Some(val)) = self.expected.next() {
                    assert_eq!(
                        val, output,
                        "Expected value {:?} but got {:?} at time: {}",
                        val, output, input.time
                    );
                }
            }
        }
        self.initialized = true;
        self.clk = clock;
    }
}

pub fn value_check_synchronous<S: Synchronous>(
    expected: impl IntoIterator<Item = Option<S::O>> + 'static,
) -> Box<dyn SynchronousValidation<S>>
where
    S::O: std::fmt::Debug,
{
    Box::new(ValueCheck {
        clk: Clock::init(),
        initialized: false,
        expected: expected.into_iter().fuse(),
    })
}
