use crate::{Digital, TimedSample, timed_sample};

/// An iterator that produces samples at uniform time intervals.
///
/// The `Uniform` iterator takes an input iterator of digital samples of type `S` and a period (in time units).
/// It produces `TimedSample<S>` items where each sample is spaced by the specified period.
///
pub struct Uniform<I, S>
where
    S: Digital,
{
    input: I,
    period: u64,
    index: u64,
    marker: std::marker::PhantomData<S>,
}

impl<I, S> Clone for Uniform<I, S>
where
    I: Clone,
    S: Digital,
{
    fn clone(&self) -> Self {
        Uniform {
            input: self.input.clone(),
            period: self.period,
            index: self.index,
            marker: std::marker::PhantomData,
        }
    }
}

impl<I, S> Iterator for Uniform<I, S>
where
    I: Iterator<Item = S>,
    S: Digital,
{
    type Item = TimedSample<S>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.input.next() {
            Some(sample) => {
                let time = self.index * self.period;
                self.index += 1;
                Some(timed_sample(time, sample))
            }
            None => None,
        }
    }
}

/// Creates a `Uniform` iterator that produces samples at uniform time intervals.
///
/// # Example
///
/// ```rust
/// # use rhdl_bits::alias::b4;
/// # use rhdl_core::sim::uniform::uniform;
///
/// let samples = vec![b4(1), b4(2), b4(3)];
/// let uniform_samples = uniform(samples.into_iter(), 100);
/// for (i, timed_sample) in uniform_samples.enumerate() {
///     assert_eq!(timed_sample.time, i as u64 * 100);
///     assert_eq!(timed_sample.value.raw(), (i + 1) as u128);
/// }
/// ```
pub fn uniform<I, S>(input: I, period: u64) -> Uniform<I, S>
where
    I: Iterator<Item = S>,
    S: Digital,
{
    Uniform {
        input,
        period,
        index: 0,
        marker: std::marker::PhantomData,
    }
}

/// Extension trait to add a `uniform` method to any iterator of digital samples.
///
/// This trait provides a convenient way to create a `Uniform` iterator from an existing iterator
/// of digital samples by specifying the desired period between samples.
///
/// # Example
///
/// ```rust
/// # use rhdl_bits::alias::b4;
/// # use rhdl_core::sim::uniform::UniformExt;
///
/// let samples = (1..).take(5).map(b4);
/// let uniform_samples = samples.uniform(100);
/// for (i, timed_sample) in uniform_samples.enumerate() {
///     assert_eq!(timed_sample.time, i as u64 * 100);
///     assert_eq!(timed_sample.value.raw(), (i + 1) as u128);
/// }
/// ```
pub trait UniformExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    fn uniform(self, period: u64) -> Uniform<Self::IntoIter, Q>;
}

impl<I, Q> UniformExt<Q> for I
where
    I: IntoIterator<Item = Q>,
    Q: Digital,
{
    fn uniform(self, period: u64) -> Uniform<Self::IntoIter, Q> {
        uniform(self.into_iter(), period)
    }
}
