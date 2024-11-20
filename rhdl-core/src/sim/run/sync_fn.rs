use crate::{
    clock::clock, clock_reset, sim::ResetOrData, timed_sample, trace, trace_time,
    types::reset::reset, Clock, ClockReset, Digital, Synchronous, SynchronousIO, TimedSample,
};

#[derive(Clone)]
enum State {
    Init,
    Hold,
    ClockLow,
    ClockHigh,
    Done,
}

//
// T - the Synchronous circuit being simulated
// F - the function that generates the input
// S - the state of the circuit
// I - the input to the circuit
// O - the output of the circuit
//
pub struct RunSynchronousFeedback<'a, T, F, S, I, O> {
    uut: &'a T,
    input_fn: F,
    uut_state: Option<S>,
    sample: ResetOrData<I>,
    last_output: Option<O>,
    time: u64,
    next_time: u64,
    period: u64,
    state: State,
}

impl<'a, T, F, S, I, O> Clone for RunSynchronousFeedback<'a, T, F, S, I, O>
where
    F: Clone,
    S: Clone,
    I: Clone,
    O: Clone,
{
    fn clone(&self) -> Self {
        RunSynchronousFeedback {
            uut: self.uut,
            input_fn: self.input_fn.clone(),
            uut_state: self.uut_state.clone(),
            sample: self.sample.clone(),
            last_output: self.last_output.clone(),
            time: self.time,
            period: self.period,
            next_time: self.next_time,
            state: self.state.clone(),
        }
    }
}

pub fn run_fn<T, F, S, I, O>(
    uut: &T,
    input_fn: F,
    period: u64,
) -> RunSynchronousFeedback<'_, T, F, S, I, O> {
    RunSynchronousFeedback {
        uut,
        input_fn,
        uut_state: None,
        time: 0,
        period,
        next_time: 0,
        state: State::Init,
        sample: ResetOrData::Reset,
        last_output: None,
    }
}

impl<'a, T, F, S, I, O> RunSynchronousFeedback<'a, T, F, S, I, O>
where
    T: Synchronous<S = S>,
    I: Digital,
    O: Digital,
    T: SynchronousIO<I = I, O = O>,
    F: FnMut(<T as SynchronousIO>::O) -> Option<ResetOrData<<T as SynchronousIO>::I>>,
{
    fn this_sample(&mut self, clock: Clock) -> TimedSample<(ClockReset, I, O)> {
        let uut_state = self.uut_state.get_or_insert_with(|| self.uut.init());
        trace_time(self.time);
        match self.sample {
            ResetOrData::Data(i) => {
                let cr = clock_reset(clock, reset(false));
                trace("clock", &cr.clock);
                trace("reset", &cr.reset);
                let o = self.uut.sim(cr, i, uut_state);
                self.last_output = Some(o);
                timed_sample(self.time, (cr, i, o))
            }
            ResetOrData::Reset => {
                let cr = clock_reset(clock, reset(true));
                trace("clock", &cr.clock);
                trace("reset", &cr.reset);
                let o = self.uut.sim(cr, I::init(), uut_state);
                self.last_output = Some(o);
                timed_sample(self.time, (cr, I::init(), o))
            }
        }
    }
}

impl<'a, T, F, S, I, O> Iterator for RunSynchronousFeedback<'a, T, F, S, I, O>
where
    T: Synchronous<S = S>,
    I: Digital,
    O: Digital,
    T: SynchronousIO<I = I, O = O>,
    F: FnMut(<T as SynchronousIO>::O) -> Option<ResetOrData<<T as SynchronousIO>::I>>,
{
    type Item = TimedSample<(ClockReset, <T as SynchronousIO>::I, <T as SynchronousIO>::O)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::Init => {
                if let Some(data) = (self.input_fn)(O::init()) {
                    self.sample = data;
                    self.state = State::Hold;
                    self.next_time = self.time + self.period / 2;
                    Some(self.this_sample(clock(false)))
                } else {
                    self.state = State::Done;
                    None
                }
            }
            State::ClockLow => {
                self.state = State::Hold;
                self.time = self.next_time;
                self.next_time = self.time + self.period / 2;
                Some(self.this_sample(clock(false)))
            }
            State::Hold => {
                self.state = State::ClockHigh;
                self.time = self.next_time;
                self.next_time += 1;
                Some(self.this_sample(clock(true)))
            }
            State::ClockHigh => {
                if let Some(data) = (self.input_fn)(self.last_output.unwrap_or(O::init())) {
                    self.sample = data;
                    self.state = State::ClockLow;
                    self.time = self.next_time;
                    self.next_time += self.period / 2 - 1;
                    Some(self.this_sample(clock(true)))
                } else {
                    self.state = State::Done;
                    None
                }
            }
            State::Done => None,
        }
    }
}

pub trait RunSynchronousFeedbackExt {
    fn run_fn<F>(
        &self,
        input_fn: F,
        period: u64,
    ) -> RunSynchronousFeedback<'_, Self, F, Self::S, Self::I, Self::O>
    where
        Self: Synchronous,
        F: FnMut(Self::O) -> Option<ResetOrData<Self::I>>;
}

impl<T> RunSynchronousFeedbackExt for T
where
    T: Synchronous,
{
    fn run_fn<F>(
        &self,
        input_fn: F,
        period: u64,
    ) -> RunSynchronousFeedback<
        '_,
        Self,
        F,
        <Self as Synchronous>::S,
        <Self as SynchronousIO>::I,
        <Self as SynchronousIO>::O,
    >
    where
        F: FnMut(<Self as SynchronousIO>::O) -> Option<ResetOrData<<Self as SynchronousIO>::I>>,
    {
        run_fn(self, input_fn, period)
    }
}
