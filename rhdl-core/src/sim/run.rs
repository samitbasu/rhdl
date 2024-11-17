use crate::{trace::db::TraceDBGuard, trace_init_db, trace_time, Circuit, CircuitIO, TimedSample};

pub struct Run<'a, T, I, S> {
    uut: &'a T,
    inputs: I,
    state: Option<S>,
    time: u64,
    guard: Option<(TraceDBGuard, String)>,
}

impl<'a, T, I, S> Clone for Run<'a, T, I, S>
where
    I: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Run {
            uut: self.uut,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            time: self.time,
            guard: None,
        }
    }
}

pub fn run_traced<T, I, S>(uut: &T, inputs: I, vcd: impl ToString) -> Run<'_, T, I, S> {
    Run {
        uut,
        inputs,
        state: None,
        time: 0,
        guard: Some((trace_init_db(), vcd.to_string())),
    }
}

pub fn run<T, I, S>(uut: &T, inputs: I) -> Run<'_, T, I, S> {
    Run {
        uut,
        inputs,
        state: None,
        time: 0,
        guard: None,
    }
}

impl<'a, T, I, S> Drop for Run<'a, T, I, S> {
    fn drop(&mut self) {
        if let Some((guard, vcd)) = self.guard.take() {
            let db = guard.take();
            let file = std::fs::File::create(&vcd).expect("failed to create VCD file");
            let mut writer = std::io::BufWriter::new(file);
            db.dump_vcd(&mut writer).expect("failed to write VCD file");
        }
    }
}

impl<'a, T, I, S> Iterator for Run<'a, T, I, S>
where
    T: Circuit<S = S>,
    I: Iterator<Item = TimedSample<<T as CircuitIO>::I>>,
{
    type Item = TimedSample<<T as CircuitIO>::O>;

    fn next(&mut self) -> Option<TimedSample<<T as CircuitIO>::O>> {
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
            let output = self.uut.sim(sample.value, state);
            Some(sample.map(|_| output))
        } else {
            None
        }
    }
}

pub trait RunExt<I> {
    fn run(&self, iter: I) -> impl Iterator
    where
        I: Iterator;

    fn run_traced(&self, iter: I, vcd: impl ToString) -> impl Iterator
    where
        I: Iterator;
}

impl<T, I> RunExt<I> for T
where
    T: Circuit,
    I: Iterator<Item = TimedSample<<T as CircuitIO>::I>>,
{
    fn run(&self, iter: I) -> impl Iterator {
        run(self, iter)
    }

    fn run_traced(&self, iter: I, vcd: impl ToString) -> impl Iterator {
        run_traced(self, iter, vcd)
    }
}
