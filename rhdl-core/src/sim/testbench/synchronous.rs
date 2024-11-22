use rhdl_trace_type::RTT;

use crate::{
    clock_reset,
    hdl::ast::{
        assert, assign, bit_string, component_instance, connection, declaration, delay, display,
        dump_file, dump_vars, finish, id, initial, unsigned_width, Direction, HDLKind, Module,
    },
    sim::test_module::TestModule,
    types::bit_string::BitString,
    ClockReset, Digital, RHDLError, Synchronous, SynchronousIO, TimedSample,
};

use super::TestBenchOptions;

#[derive(Clone)]
pub struct SynchronousTestBench<I: Digital, O: Digital> {
    pub samples: Vec<TimedSample<(ClockReset, I, O)>>,
}

impl<I, O> FromIterator<TimedSample<(ClockReset, I, O)>> for SynchronousTestBench<I, O>
where
    I: Digital,
    O: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TimedSample<(ClockReset, I, O)>>,
    {
        let samples = iter.into_iter().collect();
        SynchronousTestBench { samples }
    }
}

impl<I: Digital, O: Digital> SynchronousTestBench<I, O> {
    fn build_test_module(
        &self,
        hdl: &Module,
        options: &TestBenchOptions,
    ) -> Result<TestModule, RHDLError> {
        // All synchronous modules must have at least 2
        // ports (the first is clock + reset, the last is
        // the output).  They may have 3 if the circuit takes
        // input signals.
        let has_nonempty_input = hdl.ports.len() == 3;
        let output_port = if has_nonempty_input {
            &hdl.ports[2]
        } else {
            &hdl.ports[1]
        };
        if hdl.ports[0].direction != Direction::Input || hdl.ports[0].width != unsigned_width(2) {
            return Err(RHDLError::TestbenchConstructionError(
                "First port must be an input with 2 bits width".into(),
            ));
        }
        if has_nonempty_input && (I::BITS == 0) {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port mismatch".into(),
            ));
        }
        if has_nonempty_input && I::BITS != hdl.ports[1].width.len() {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port width mismatch".into(),
            ));
        }
        if output_port.direction != Direction::Output || output_port.width.len() != O::BITS {
            return Err(RHDLError::TestbenchConstructionError(
                "Output port mismatch".into(),
            ));
        }
        let arg0_connection = Some(connection(&hdl.ports[0].name, id("clock_reset")));
        let arg1_connection = (has_nonempty_input).then(|| connection(&hdl.ports[1].name, id("i")));
        let arg2_connection = Some(connection(&output_port.name, id("o")));
        let uut_name = &hdl.name;
        let instance = component_instance(
            uut_name,
            "t",
            vec![arg0_connection, arg1_connection, arg2_connection]
                .into_iter()
                .flatten()
                .collect(),
        );
        let declarations = vec![
            Some(declaration(
                HDLKind::Reg,
                "clock_reset",
                unsigned_width(ClockReset::bits()),
                None,
            )),
            (has_nonempty_input)
                .then(|| declaration(HDLKind::Reg, "i", unsigned_width(I::BITS), None)),
            Some(declaration(
                HDLKind::Wire,
                "o",
                unsigned_width(O::BITS),
                None,
            )),
            Some(declaration(
                HDLKind::Reg,
                "rust_out",
                unsigned_width(O::BITS),
                None,
            )),
        ];
        let mut test_cases = vec![];
        if let Some(vcd_file) = &options.vcd_file {
            test_cases.push(dump_file(vcd_file));
            test_cases.push(dump_vars(0));
            // Also write out an RTT file for this VCD that can be loaded
            // afterwards to provide type information for the VCD
            let rtt = RTT::TraceInfo(
                [
                    (
                        "testbench.clock_reset".to_string(),
                        ClockReset::static_trace_type(),
                    ),
                    ("testbench.i".to_string(), I::static_trace_type()),
                    ("testbench.o".to_string(), O::static_trace_type()),
                    ("testbench.rust_out".to_string(), O::static_trace_type()),
                ]
                .into_iter()
                .collect(),
            );
            std::fs::write(
                vcd_file.clone() + ".rtt",
                ron::ser::to_string(&rtt).unwrap(),
            )?;
        }
        let mut absolute_time = 0;
        for (test_case_counter, timed_entry) in self.samples.iter().enumerate() {
            let sample_time = timed_entry.time;
            let (sample_cr, sample_i, sample_o) = timed_entry.value;
            // First, we determine if at least the hold time has elapsed between the sample time and the previous recorded time
            // and ensure that we actually have an expected output and that we have passed the number of test cases to skip
            if sample_time.saturating_sub(absolute_time) > options.hold_time
                && test_case_counter > 0
                && test_case_counter >= options.skip_first_cases
            {
                test_cases.push(delay(options.hold_time));
                test_cases.push(assert(
                    id("o"),
                    id("rust_out"),
                    &format!("Test {test_case_counter} at time {absolute_time}"),
                ));
                absolute_time += options.hold_time;
            }
            test_cases.push(delay(sample_time.saturating_sub(absolute_time)));
            absolute_time = sample_time;
            let cr = clock_reset(sample_cr.clock, sample_cr.reset);
            test_cases.push(assign("clock_reset", bit_string(&cr.typed_bits().into())));
            if has_nonempty_input {
                test_cases.push(assign(
                    "i",
                    bit_string(&BitString::unsigned(sample_i.bin())),
                ));
            }
            test_cases.push(assign(
                "rust_out",
                bit_string(&BitString::unsigned(sample_o.bin())),
            ));
        }
        test_cases.push(display("TESTBENCH OK", vec![]));
        test_cases.push(finish());
        let module = Module {
            name: "testbench".into(),
            description: "Testbench for synchronous module".into(),
            declarations: declarations.into_iter().flatten().collect(),
            statements: vec![instance, initial(test_cases)],
            submodules: vec![hdl.clone()],
            ..Default::default()
        };
        Ok(module.into())
    }

    pub fn rtl<T>(&self, uut: &T, options: &TestBenchOptions) -> Result<TestModule, RHDLError>
    where
        T: Synchronous,
        T: SynchronousIO<I = I, O = O>,
    {
        let module = uut.hdl("uut")?.as_module();
        self.build_test_module(&module, options)
    }
    pub fn flow_graph<T>(
        &self,
        uut: &T,
        options: &TestBenchOptions,
    ) -> Result<TestModule, RHDLError>
    where
        T: Synchronous,
        T: SynchronousIO<I = I, O = O>,
    {
        let module = uut.flow_graph("uut")?.hdl("dut")?;
        self.build_test_module(&module, options)
    }
}
