use crate::{
    hdl::ast::{
        component_instance, connection, declaration, dump_file, dump_vars, id, unsigned_width,
        Direction, HDLKind, Module,
    },
    testbench::test_module::TestModule,
    ClockReset, Digital, RHDLError, Synchronous, TimedSample,
};

use super::TestBenchOptions;

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

#[derive(Copy, Clone, PartialEq, Debug)]
enum ResetState {
    Initial,
    InReset,
    ResetDone,
}

impl<I: Digital, O: Digital> TestBench<I, O> {
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
        }
        let mut reset_state = ResetState::Initial;
        let mut extra_delay = 0;
        let mut absolute_time = 0;
        for (test_case_counter, timed_entry) in self.samples.iter().enumerate() {
            test_cases.push(delay(
                (timed_entry.delay as usize).saturating_sub(extra_delay),
            ));
            absolute_time += timed_entry.delay as usize;
            let cr = clock_reset(timed_entry.clock, timed_entry.reset);
            test_cases.push(assign("clock_reset", bit_string(&cr.typed_bits().into())));
            let input = timed_entry.input.clone();
            if has_nonempty_input {
                test_cases.push(assign("i", bit_string(&BitString::unsigned(input))));
            }
            let output = timed_entry.output.clone();
            test_cases.push(assign("rust_out", bit_string(&BitString::unsigned(output))));
            let reset_bit_enabled = cr.reset.any();
            reset_state = match (reset_state, reset_bit_enabled) {
                (ResetState::Initial, true) => ResetState::InReset,
                (ResetState::InReset, false) => ResetState::ResetDone,
                (ResetState::ResetDone, true) => ResetState::InReset,
                (ResetState::ResetDone, false) => ResetState::ResetDone,
                _ => reset_state,
            };
            if (reset_state == ResetState::ResetDone)
                && !cr.clock.raw()
                && test_case_counter >= options.skip_first_cases
            {
                test_cases.push(delay(options.hold_time as usize));
                extra_delay = options.hold_time as usize;
                test_cases.push(assert(
                    id("o"),
                    id("rust_out"),
                    &format!("Test {test_case_counter} at time {absolute_time}"),
                ));
            }
        }
        if reset_state != ResetState::Initial {
            test_cases.push(display("TESTBENCH OK", vec![]));
        } else {
            test_cases.push(display("TESTBENCH FAILED - NO RESET PROVIDED", vec![]));
        }
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

    pub fn rtl<T: Synchronous>(self, uut: &T) -> Result<TestModule, RHDLError> {}
    pub fn flowgraph<T: Synchronous>(self, uut: &T) -> Result<TestModule, RHDLError> {}
}
