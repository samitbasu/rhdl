use rhdl_trace_type::RTT;

use crate::rhdl_core::{
    hdl::ast::{
        assert, assign, bit_string, component_instance, connection, declaration, delay, display,
        dump_file, dump_vars, finish, id, initial, unsigned_width, Direction, HDLKind, Module,
    },
    sim::test_module::TestModule,
    types::bit_string::BitString,
    Circuit, CircuitIO, Digital, RHDLError, TimedSample,
};

use super::TestBenchOptions;

#[derive(Clone)]
pub struct TestBench<I: Digital, O: Digital> {
    pub samples: Vec<TimedSample<(I, O)>>,
}

impl<I, O> FromIterator<TimedSample<(I, O)>> for TestBench<I, O>
where
    I: Digital,
    O: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TimedSample<(I, O)>>,
    {
        let samples = iter.into_iter().collect();
        TestBench { samples }
    }
}

impl<I: Digital, O: Digital> TestBench<I, O> {
    fn build_test_module(
        &self,
        hdl: &Module,
        options: &TestBenchOptions,
    ) -> Result<TestModule, RHDLError> {
        // Asynchronous modules may have either 1 or 2 ports.
        // If the module has 2 ports, the first port is the input
        let has_nonempty_input = hdl.ports.len() == 2;
        let output_port = if has_nonempty_input {
            &hdl.ports[1]
        } else {
            &hdl.ports[0]
        };
        if has_nonempty_input && I::BITS == 0 {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port mismatch".into(),
            ));
        }
        if has_nonempty_input && I::BITS != hdl.ports[0].width.len() {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port width mismatch".into(),
            ));
        }
        if output_port.direction != Direction::Output || output_port.width.len() != O::BITS {
            return Err(RHDLError::TestbenchConstructionError(
                "Output port mismatch".into(),
            ));
        }
        let arg1_connection = (has_nonempty_input).then(|| connection(&hdl.ports[0].name, id("i")));
        let arg2_connection = Some(connection(&output_port.name, id("o")));
        let uut_name = &hdl.name;
        let instance = component_instance(
            uut_name,
            "t",
            vec![arg1_connection, arg2_connection]
                .into_iter()
                .flatten()
                .collect(),
        );
        let declarations = vec![
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
            let (sample_i, sample_o) = timed_entry.value;
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
            description: "Testbench for asynchronous module".into(),
            declarations: declarations.into_iter().flatten().collect(),
            statements: vec![instance, initial(test_cases)],
            submodules: vec![hdl.clone()],
            ..Default::default()
        };
        Ok(module.into())
    }

    pub fn rtl<T>(&self, uut: &T, options: &TestBenchOptions) -> Result<TestModule, RHDLError>
    where
        T: Circuit,
        T: CircuitIO<I = I, O = O>,
    {
        let hdl = uut.hdl("uut")?.as_module();
        self.build_test_module(&hdl, options)
    }

    pub fn ntl<T>(&self, uut: &T, options: &TestBenchOptions) -> Result<TestModule, RHDLError>
    where
        T: Circuit,
        T: CircuitIO<I = I, O = O>,
    {
        let module = crate::rhdl_core::ntl::hdl::generate_hdl("dut", &uut.descriptor("uut")?.ntl)?;
        self.build_test_module(&module, options)
    }
}
