use crate::{
    Digital, TimedSample,
    trace2::{session::Session, trace_sample::TraceSample},
};

#[derive(Default)]
pub struct TracedIterator<I> {
    session: Session,
    inner: I,
}

impl<T: Digital, I: Iterator<Item = TimedSample<T>>> Iterator for TracedIterator<I> {
    type Item = TraceSample<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| self.session.traced(x))
    }
}

impl<I> TracedIterator<I> {
    pub fn new(session: Session, inner: I) -> Self {
        Self { session, inner }
    }
}

pub fn traced<T: Digital, I: Iterator<Item = TimedSample<T>>>(
    session: Session,
    iter: I,
) -> TracedIterator<I> {
    TracedIterator::new(session.clone(), iter)
}

/// Extension trait to add `.traced()` to any iterator that
/// yields `TimedSample<T>` items.
pub trait TraceSessionExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    fn traced(self) -> TracedIterator<Self>;
}

impl<I, Q> TraceSessionExt<Q> for I
where
    I: IntoIterator<Item = TimedSample<Q>>,
    Q: Digital,
{
    fn traced(self) -> TracedIterator<Self> {
        let session = Session::default();
        TracedIterator::new(session, self)
    }
}

#[derive(Default)]
pub struct UntracedIterator<I> {
    inner: I,
}

impl<T: Digital, I: Iterator<Item = TimedSample<T>>> Iterator for UntracedIterator<I> {
    type Item = TraceSample<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| TraceSample::untraced(x))
    }
}

pub fn untraced<T: Digital, I: Iterator<Item = TimedSample<T>>>(iter: I) -> UntracedIterator<I> {
    UntracedIterator { inner: iter }
}

/// Extension trait to add `.untraced()` to any iterator that
/// yields `TimedSample<T>` items.
pub trait UntracedExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    fn untraced(self) -> UntracedIterator<Self>;
}

impl<I, Q> UntracedExt<Q> for I
where
    I: IntoIterator<Item = TimedSample<Q>>,
    Q: Digital,
{
    fn untraced(self) -> UntracedIterator<Self> {
        UntracedIterator { inner: self }
    }
}
