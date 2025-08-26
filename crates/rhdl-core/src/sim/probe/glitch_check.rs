use crate::rhdl_core::{Clock, Digital, TimedSample};

pub struct GlitchCheck<T, I, F> {
    clk: Clock,
    prev_val: T,
    iter: I,
    func: F,
    initialized: bool,
    edge_time: u64,
    tolerance: u64,
}

impl<T, I, F> Clone for GlitchCheck<T, I, F>
where
    I: Clone,
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        GlitchCheck {
            clk: self.clk,
            prev_val: self.prev_val.clone(),
            iter: self.iter.clone(),
            func: self.func.clone(),
            initialized: self.initialized,
            edge_time: self.edge_time,
            tolerance: self.tolerance,
        }
    }
}

pub fn glitch_check<T, I, F>(stream: I, clock_fn: F) -> GlitchCheck<T, I, F>
where
    T: Digital,
{
    GlitchCheck {
        clk: Clock::default(),
        prev_val: T::dont_care(),
        iter: stream,
        func: clock_fn,
        initialized: false,
        edge_time: 0,
        tolerance: 1,
    }
}

impl<T, I, F, S> Iterator for GlitchCheck<T, I, F>
where
    T: Digital + std::fmt::Debug,
    I: Iterator<Item = TimedSample<S>>,
    F: Fn(&TimedSample<S>) -> (Clock, T),
    S: Digital,
{
    type Item = TimedSample<S>;

    fn next(&mut self) -> Option<TimedSample<S>> {
        if !self.initialized {
            if let Some(sample) = self.iter.next() {
                (self.clk, self.prev_val) = (self.func)(&sample);
                self.initialized = true;
                return Some(sample);
            } else {
                return None;
            }
        }
        if let Some(sample) = self.iter.next() {
            let (clk, value) = (self.func)(&sample);
            let pos_edge = clk.raw() && !self.clk.raw();
            let output_changed = value != self.prev_val;
            if output_changed && !pos_edge && sample.time - self.edge_time > self.tolerance {
                panic!(
                    "Glitch detected at time: {time}, sample = {value:?}, prev_val = {prev:?}",
                    time = sample.time,
                    prev = self.prev_val
                );
            }
            if pos_edge {
                self.edge_time = sample.time;
            }
            self.clk = clk;
            self.prev_val = value;
            Some(sample)
        } else {
            None
        }
    }
}
