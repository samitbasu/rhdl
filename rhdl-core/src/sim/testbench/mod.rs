pub mod asynchronous;
pub mod synchronous;

#[derive(Debug, Clone, Default)]
pub struct TestBenchOptions {
    pub vcd_file: Option<String>,
    pub skip_first_cases: usize,
    pub hold_time: u64,
}

impl TestBenchOptions {
    fn vcd(self, vcd_file: &str) -> Self {
        Self {
            vcd_file: Some(vcd_file.into()),
            ..self
        }
    }
    fn skip(self, skip_first_cases: usize) -> Self {
        Self {
            skip_first_cases,
            ..self
        }
    }
    fn hold_time(self, hold_time: u64) -> Self {
        Self { hold_time, ..self }
    }
}
