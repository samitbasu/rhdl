use crate::{
    Clock, ClockReset, Digital, Synchronous, SynchronousIO, TimedSample,
    clock::clock,
    clock_reset,
    sim::ResetOrData,
    timed_sample, trace,
    trace::{
        page::{set_trace_page, take_trace_page},
        session::Session,
        trace_sample::TracedSample,
    },
    types::reset::reset,
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
    session: Session,
}

impl<T, F, S, I, O> Clone for RunSynchronousFeedback<'_, T, F, S, I, O>
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
            session: self.session.clone(),
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
        session: Session::default(),
    }
}

impl<T, F, S, I, O> RunSynchronousFeedback<'_, T, F, S, I, O>
where
    T: Synchronous<S = S>,
    I: Digital,
    O: Digital,
    T: SynchronousIO<I = I, O = O>,
    F: FnMut(<T as SynchronousIO>::O) -> Option<ResetOrData<<T as SynchronousIO>::I>>,
{
    fn this_sample(&mut self, clock: Clock) -> TracedSample<(ClockReset, I), O> {
        let uut_state = self.uut_state.get_or_insert_with(|| self.uut.init());
        match self.sample {
            ResetOrData::Data(i) => {
                let cr = clock_reset(clock, reset(false));
                set_trace_page(Some(self.session.page()));
                trace("clock", &cr.clock);
                trace("reset", &cr.reset);
                let o = self.uut.sim(cr, i, uut_state);
                self.last_output = Some(o);
                TracedSample {
                    time: self.time,
                    page: take_trace_page(),
                    input: (cr, i),
                    output: o,
                }
            }
            ResetOrData::Reset => {
                let cr = clock_reset(clock, reset(true));
                set_trace_page(Some(self.session.page()));
                trace("clock", &cr.clock);
                trace("reset", &cr.reset);
                let o = self.uut.sim(cr, I::dont_care(), uut_state);
                self.last_output = Some(o);
                TracedSample {
                    time: self.time,
                    page: take_trace_page(),
                    input: (cr, I::dont_care()),
                    output: o,
                }
            }
        }
    }
}

impl<T, F, S, I, O> Iterator for RunSynchronousFeedback<'_, T, F, S, I, O>
where
    T: Synchronous<S = S>,
    I: Digital,
    O: Digital,
    T: SynchronousIO<I = I, O = O>,
    F: FnMut(<T as SynchronousIO>::O) -> Option<ResetOrData<<T as SynchronousIO>::I>>,
{
    type Item = TracedSample<(ClockReset, <T as SynchronousIO>::I), <T as SynchronousIO>::O>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::Init => {
                if let Some(data) = (self.input_fn)(O::dont_care()) {
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
                if let Some(data) = (self.input_fn)(self.last_output.unwrap_or(O::dont_care())) {
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
