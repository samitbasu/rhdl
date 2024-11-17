use crate::{Clock, Digital, TimedSample};

pub struct GlitchFree<T, I, F> {
    clk: Clock,
    prev_val: T,
    iter: I,
    func: F,
    initialized: bool,
}

impl<T, I, F> Clone for GlitchFree<T, I, F>
where
    I: Clone,
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        GlitchFree {
            clk: self.clk,
            prev_val: self.prev_val.clone(),
            iter: self.iter.clone(),
            func: self.func.clone(),
            initialized: self.initialized,
        }
    }
}

pub fn glitch_free<T, I, F>(stream: I, clock_fn: F) -> GlitchFree<T, I, F>
where
    T: Digital,
{
    GlitchFree {
        clk: Clock::default(),
        prev_val: T::init(),
        iter: stream,
        func: clock_fn,
        initialized: false,
    }
}

impl<T, I, F> Iterator for GlitchFree<T, I, F>
where
    T: Digital + std::fmt::Debug,
    I: Iterator<Item = TimedSample<T>>,
    F: Fn(&TimedSample<T>) -> Clock,
{
    type Item = TimedSample<T>;

    fn next(&mut self) -> Option<TimedSample<T>> {
        if !self.initialized {
            if let Some(sample) = self.iter.next() {
                self.clk = (self.func)(&sample);
                self.prev_val = sample.value;
                self.initialized = true;
                return Some(sample);
            } else {
                return None;
            }
        }
        if let Some(sample) = self.iter.next() {
            let clk = (self.func)(&sample);
            let pos_edge = clk.raw() && !self.clk.raw();
            let output_changed = sample.value != self.prev_val;
            if output_changed && !pos_edge {
                panic!(
                    "Glitch detected at time: {time}, sample = {sample:?}, prev_val = {prev:?}",
                    time = sample.time,
                    prev = self.prev_val
                );
            }
            self.clk = clk;
            self.prev_val = sample.value;
            Some(sample)
        } else {
            None
        }
    }
}
