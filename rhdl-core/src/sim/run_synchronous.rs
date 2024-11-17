use crate::{
    trace, trace::db::TraceDBGuard, trace_init_db, trace_time, ClockReset, Synchronous,
    SynchronousIO, TimedSample,
};

pub struct RunSynchronous<'a, T, I, S> {
    uut: &'a T,
    inputs: I,
    state: Option<S>,
    time: u64,
    guard: Option<(TraceDBGuard, String)>,
}

impl<'a, T, I, S> Clone for RunSynchronous<'a, T, I, S>
where
    I: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        RunSynchronous {
            uut: self.uut,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            time: self.time,
            guard: None,
        }
    }
}

pub fn run_traced_synchronous<T, I, S>(
    uut: &T,
    inputs: I,
    vcd: impl ToString,
) -> RunSynchronous<'_, T, I, S> {
    RunSynchronous {
        uut,
        inputs,
        state: None,
        time: 0,
        guard: Some((trace_init_db(), vcd.to_string())),
    }
}

pub fn run_synchronous<T, I, S>(uut: &T, inputs: I) -> RunSynchronous<'_, T, I, S> {
    RunSynchronous {
        uut,
        inputs,
        state: None,
        time: 0,
        guard: None,
    }
}

impl<'a, T, I, S> Drop for RunSynchronous<'a, T, I, S> {
    fn drop(&mut self) {
        if let Some((guard, vcd)) = self.guard.take() {
            let db = guard.take();
            let file = std::fs::File::create(&vcd).expect("failed to create VCD file");
            let mut writer = std::io::BufWriter::new(file);
            db.dump_vcd(&mut writer).expect("failed to write VCD file");
        }
    }
}

impl<'a, T, I, S> Iterator for RunSynchronous<'a, T, I, S>
where
    T: Synchronous<S = S>,
    I: Iterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    type Item = TimedSample<<T as SynchronousIO>::O>;

    fn next(&mut self) -> Option<TimedSample<<T as SynchronousIO>::O>> {
        // Get a mutable borrow to the state.  If the state is None
        // then initialize it first.
        let state = self.state.get_or_insert_with(|| self.uut.init());
        if let Some(sample) = self.inputs.next() {
            assert!(
                sample.time >= self.time,
                "input time must be non-decreasing"
            );
            self.time = sample.time;
            trace_time(sample.time);
            trace("clock", &sample.value.0.clock);
            trace("reset", &sample.value.0.reset);
            let output = self.uut.sim(sample.value.0, sample.value.1, state);
            Some(sample.map(|_| output))
        } else {
            None
        }
    }
}

pub trait RunSynchronousExt<I> {
    fn run(&self, iter: I) -> impl Iterator
    where
        I: Iterator;

    fn run_traced(&self, iter: I, vcd: impl ToString) -> impl Iterator
    where
        I: Iterator;
}

impl<T, I> RunSynchronousExt<I> for T
where
    T: Synchronous,
    I: Iterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    fn run(&self, iter: I) -> impl Iterator {
        run_synchronous(self, iter)
    }

    fn run_traced(&self, iter: I, vcd: impl ToString) -> impl Iterator {
        run_traced_synchronous(self, iter, vcd)
    }
}
