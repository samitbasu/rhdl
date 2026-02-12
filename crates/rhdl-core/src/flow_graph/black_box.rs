use crate::{
    Circuit, CircuitIO, ClockReset, Digital, RHDLError, ScopedName, Synchronous, SynchronousIO,
    flow_graph::{FlowGraph, builder::Builder},
};

pub fn build_circuit_blackbox<C: Circuit>(
    scoped_name: &ScopedName,
) -> Result<FlowGraph, RHDLError> {
    let mut builder = Builder::new(&scoped_name.to_string());
    let input_kind = <<C as CircuitIO>::I as Digital>::static_kind();
    let output_kind = <<C as CircuitIO>::O as Digital>::static_kind();
    builder.add_input_port(input_kind, 0);
    builder.add_output_port(output_kind);
    Ok(builder.build())
}

pub fn build_synchronous_blackbox<C: Synchronous>(
    scoped_name: &ScopedName,
) -> Result<FlowGraph, RHDLError> {
    let mut builder = Builder::new(&scoped_name.to_string());
    let input_kind = <<C as SynchronousIO>::I as Digital>::static_kind();
    let output_kind = <<C as SynchronousIO>::O as Digital>::static_kind();
    let cr_kind = ClockReset::static_kind();
    builder.add_input_port(cr_kind, 0);
    builder.add_input_port(input_kind, 1);
    builder.add_output_port(output_kind);
    Ok(builder.build())
}
