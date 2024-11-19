pub mod asynchronous;
pub mod kernel;
pub mod synchronous;

#[derive(Debug, Clone)]
pub struct TestBenchOptions {
    vcd_file: Option<String>,
    skip_first_cases: usize,
    hold_time: u64,
}

impl TestBenchOptions {
    pub fn vcd(self, vcd_file: &str) -> Self {
        Self {
            vcd_file: Some(vcd_file.into()),
            ..self
        }
    }
    pub fn skip(self, skip_first_cases: usize) -> Self {
        Self {
            skip_first_cases,
            ..self
        }
    }
    pub fn hold_time(self, hold_time: u64) -> Self {
        assert!(hold_time > 0, "hold_time must be positive");
        Self { hold_time, ..self }
    }
}

impl Default for TestBenchOptions {
    fn default() -> Self {
        Self {
            vcd_file: None,
            skip_first_cases: 0,
            hold_time: 1,
        }
    }
}
