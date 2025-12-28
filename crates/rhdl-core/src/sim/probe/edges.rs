use crate::{Digital, TimedSample, trace::trace_sample::TracedSample};

pub struct EdgeTime<T, I, F> {
    iter: I,
    func: F,
    initialized: bool,
    prev_val: T,
}

impl<T, I, F> Clone for EdgeTime<T, I, F>
where
    I: Clone,
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        EdgeTime {
            iter: self.iter.clone(),
            func: self.func.clone(),
            initialized: self.initialized,
            prev_val: self.prev_val.clone(),
        }
    }
}

pub fn edge_time<T, I, F>(stream: I, data_fn: F) -> EdgeTime<T, I, F>
where
    T: Digital,
{
    EdgeTime {
        iter: stream,
        func: data_fn,
        initialized: false,
        prev_val: T::dont_care(),
    }
}

impl<T, I, F, S, U> Iterator for EdgeTime<T, I, F>
where
    T: Digital,
    I: Iterator<Item = TracedSample<S, U>>,
    F: Fn(&TracedSample<S, U>) -> T,
    S: Digital,
    U: Digital,
{
    type Item = TracedSample<S, U>;
    fn next(&mut self) -> Option<TracedSample<S, U>> {
        loop {
            if !self.initialized {
                if let Some(sample) = self.iter.next() {
                    self.prev_val = (self.func)(&sample);
                    self.initialized = true;
                } else {
                    return None;
                }
            }
            if let Some(sample) = self.iter.next() {
                let val = (self.func)(&sample);
                if val != self.prev_val {
                    self.prev_val = val;
                    return Some(sample);
                }
            } else {
                return None;
            }
        }
    }
}
