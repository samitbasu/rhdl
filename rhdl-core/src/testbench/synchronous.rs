use crate::{
    clock_reset,
    hdl::{
        ast::{
            assert, assign, bit_string, component_instance, connection, declaration, delay,
            display, dump_file, dump_vars, finish, id, initial, unsigned_width, Direction, HDLKind,
            Module,
        },
        formatter,
    },
    sim::waveform::SynchronousWaveform,
    types::bit_string::BitString,
    waveform_synchronous, ClockReset, Digital, RHDLError, Synchronous, TimedSample,
};

use super::{test_module::TestModule, TraceOptions};

#[derive(Copy, Clone, PartialEq, Debug)]
enum ResetState {
    Initial,
    InReset,
    ResetDone,
}

fn build_test_module_from_synchronous_waveform(
    modules: &[Module],
    waveform: &SynchronousWaveform,
    options: &TraceOptions,
) -> Result<TestModule, RHDLError> {
    // All synchronous modules must have at least 2
    // ports (the first is clock + reset, the last is
    // the output).  They may have 3 if the circuit takes
    // input signals.
    if modules.is_empty() {
        return Err(RHDLError::TestbenchConstructionError(
            "No modules provided".into(),
        ));
    }
    let hdl = &modules[0];
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
    if has_nonempty_input == waveform.input_kind.is_empty() {
        return Err(RHDLError::TestbenchConstructionError(
            "Input port mismatch".into(),
        ));
    }
    if has_nonempty_input && waveform.input_kind.bits() != hdl.ports[1].width.len() {
        return Err(RHDLError::TestbenchConstructionError(
            "Input port width mismatch".into(),
        ));
    }
    if output_port.direction != Direction::Output
        || output_port.width.len() != waveform.output_kind.bits()
    {
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
        (has_nonempty_input).then(|| {
            declaration(
                HDLKind::Reg,
                "i",
                unsigned_width(waveform.input_kind.bits()),
                None,
            )
        }),
        Some(declaration(
            HDLKind::Wire,
            "o",
            unsigned_width(waveform.output_kind.bits()),
            None,
        )),
        Some(declaration(
            HDLKind::Reg,
            "rust_out",
            unsigned_width(waveform.output_kind.bits()),
            None,
        )),
    ];
    let mut test_cases = if let Some(filename) = &options.vcd {
        vec![dump_file(filename), dump_vars(0)]
    } else {
        vec![]
    };
    let mut reset_state = ResetState::Initial;
    for (test_case_counter, timed_entry) in waveform.entries.iter().enumerate() {
        test_cases.push(delay(timed_entry.delay as usize));
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
        if (reset_state == ResetState::ResetDone) && !cr.clock.raw() && options.assertions_enabled {
            test_cases.push(delay(0));
            test_cases.push(assert(id("o"), id("rust_out"), test_case_counter));
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
        ..Default::default()
    };
    let modules_as_verilog = modules
        .iter()
        .map(formatter::module)
        .collect::<Vec<String>>()
        .join("\n");
    Ok(TestModule {
        testbench: module.as_verilog() + &modules_as_verilog,
        num_cases: 0,
    })
}

pub fn test_synchronous_hdl<T: Synchronous>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<(ClockReset, T::I)>>,
    opts: TraceOptions,
) -> Result<(), RHDLError> {
    // Get a waveform for this circuit
    let waveform = waveform_synchronous(uut, inputs);
    // Construct a RTL-based test bench
    let rtl_mod = uut.hdl("uut")?.as_modules();
    let tm1 = build_test_module_from_synchronous_waveform(&rtl_mod, &waveform, &opts)?;
    tm1.run_iverilog()?;
    // Construct a flowgraph-based test bench
    let fg = uut.flow_graph("uut")?.hdl("uut")?;
    let tm1 = build_test_module_from_synchronous_waveform(&[fg], &waveform, &opts)?;
    tm1.run_iverilog()?;

    Ok(())
}
