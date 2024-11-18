use crate::{
    testbench::test_module::TestModule, ClockReset, Digital, RHDLError, Synchronous, TimedSample,
};

#[derive(Clone)]
pub struct TestBench<I: Digital, O: Digital> {
    pub samples: Vec<TimedSample<(ClockReset, I, O)>>,
}

impl<I, O> FromIterator<TimedSample<(ClockReset, I, O)>> for TestBench<I, O>
where
    I: Digital,
    O: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TimedSample<(ClockReset, I, O)>>,
    {
        let samples = iter.into_iter().collect();
        TestBench { samples }
    }
}

impl<I: Digital, O: Digital> TestBench<I, O> {
    pub fn rtl<T: Synchronous>(self, uut: &T) -> Result<TestModule, RHDLError> {}
    pub fn flowgraph<T: Synchronous>(self, uut: &T) -> Result<TestModule, RHDLError> {}
}
