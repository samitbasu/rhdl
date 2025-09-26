use crate::{Digital, TimedSample, timed_sample};

pub fn uniform<I, S>(input: I, period: u64) -> impl Iterator<Item = TimedSample<S>>
where
    I: Iterator<Item = S>,
    S: Digital,
{
    input
        .enumerate()
        .map(move |(i, sample)| timed_sample(i as u64 * period, sample))
}

pub trait UniformExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    fn uniform(self, period: u64) -> impl Iterator<Item = TimedSample<Q>>;
}

impl<I, Q> UniformExt<Q> for I
where
    I: IntoIterator<Item = Q>,
    Q: Digital,
{
    fn uniform(self, period: u64) -> impl Iterator<Item = TimedSample<Q>> {
        uniform(self.into_iter(), period)
    }
}
