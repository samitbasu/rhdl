pub mod asynchronous;
pub mod kernel;
pub mod synchronous;
pub mod test_module;

#[derive(Clone, PartialEq, Debug)]
pub struct TraceOptions {
    pub vcd: Option<String>,
    pub assertions_enabled: bool,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            vcd: None,
            assertions_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestModuleOptions {
    pub vcd_file: Option<String>,
    pub skip_first_cases: usize,
    pub hold_time: u64,
    pub flow_graph_level: bool,
}
