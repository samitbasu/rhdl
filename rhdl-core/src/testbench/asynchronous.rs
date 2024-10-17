use crate::{
    hdl::{
        ast::{
            assert, assign, bit_string, component_instance, connection, declaration, delay,
            display, finish, id, initial, unsigned_width, Direction, HDLKind, Module,
        },
        formatter,
    },
    sim::waveform::{waveform, AsynchronousWaveform},
    types::bit_string::BitString,
    Circuit, RHDLError, TimedSample,
};

use super::test_module::TestModule;

fn build_test_module_from_waveform(
    modules: &[Module],
    waveform: &AsynchronousWaveform,
) -> Result<TestModule, RHDLError> {
    // All synchronous modules must have at least 1
    // ports.  They may have 2 if hte circuit takes input signals.
    if modules.is_empty() {
        return Err(RHDLError::TestbenchConstructionError(
            "No modules provided".into(),
        ));
    }
    let hdl = &modules[0];
    let has_nonempty_input = hdl.ports.len() == 2;
    let output_port = if has_nonempty_input {
        &hdl.ports[1]
    } else {
        &hdl.ports[0]
    };
    if has_nonempty_input == waveform.input_kind.is_empty() {
        return Err(RHDLError::TestbenchConstructionError(
            "Input port mismatch".into(),
        ));
    }
    if has_nonempty_input && waveform.input_kind.bits() != hdl.ports[0].width.len() {
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
    let mut test_cases = vec![];
    for (test_case_counter, timed_entry) in waveform.entries.iter().enumerate() {
        test_cases.push(delay(timed_entry.delay as usize));
        let input = timed_entry.input.clone();
        if has_nonempty_input {
            test_cases.push(assign("i", bit_string(&BitString::unsigned(input))));
        }
        let output = timed_entry.output.clone();
        test_cases.push(assign("rust_out", bit_string(&BitString::unsigned(output))));
        test_cases.push(delay(0));
        test_cases.push(assert(id("o"), id("rust_out"), test_case_counter));
    }
    test_cases.push(display("TESTBENCH OK", vec![]));
    test_cases.push(finish());
    let module = Module {
        name: "testbench".into(),
        ports: vec![],
        declarations: declarations.into_iter().flatten().collect(),
        statements: vec![instance, initial(test_cases)],
        functions: vec![],
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

pub fn test_asynchronous_hdl<T: Circuit>(
    uut: &T,
    inputs: impl Iterator<Item = TimedSample<T::I>>,
) -> Result<(), RHDLError> {
    // Get a waveform for this circuit
    let wav = waveform(uut, inputs);
    let rtl_mod = uut.hdl()?.as_modules();
    let tm1 = build_test_module_from_waveform(&rtl_mod, &wav)?;
    tm1.run_iverilog()?;
    let fg = uut.flow_graph()?.hdl("uut")?;
    let tm1 = build_test_module_from_waveform(&[fg], &wav)?;
    tm1.run_iverilog()?;
    Ok(())
}
