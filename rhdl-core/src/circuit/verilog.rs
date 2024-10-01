use crate::compile_design;
use crate::compiler::codegen::verilog::generate_verilog;
use crate::error::RHDLError;
use crate::types::digital::Digital;
use crate::types::path::{bit_range, Path};
use crate::types::tristate::Tristate;
use crate::verilog::ast::{
    connection, continuous_assignment, declaration, function_call, id, index, index_range, port,
    unsigned_width, ComponentInstance, Direction, Kind, Module, Statement,
};

use super::{
    circuit_descriptor::CircuitDescriptor, circuit_impl::Circuit, hdl_descriptor::HDLDescriptor,
};

type Result<T> = std::result::Result<T, RHDLError>;

pub fn root_verilog<C: Circuit>(t: &C) -> Result<HDLDescriptor> {
    // Start with the module declaration for the circuit.
    let descriptor = t.descriptor()?;
    let input_bits = C::I::bits();
    let outputs = C::O::bits();

    let module_name = &descriptor.unique_name;
    // module top(input wire clk, input wire[0:0] top_in, output reg[3:0] top_out);
    let mut module = Module::default();
    module.name = module_name.clone();
    module.ports.push(port(
        "i",
        Direction::Input,
        Kind::Wire,
        unsigned_width(input_bits),
    ));
    module.ports.push(port(
        "o",
        Direction::Output,
        Kind::Wire,
        unsigned_width(outputs),
    ));
    if C::Z::N != 0 {
        module.ports.push(port(
            "io",
            Direction::Inout,
            Kind::Wire,
            unsigned_width(C::Z::N),
        ));
    }

    let o_d_bits = C::O::bits() + C::D::bits();
    // Next declare the D and Q wires
    module.declarations.push(declaration(
        Kind::Wire,
        "od",
        unsigned_width(o_d_bits),
        None,
    ));
    module.declarations.push(declaration(
        Kind::Wire,
        "d",
        unsigned_width(C::D::bits()),
        None,
    ));
    module.declarations.push(declaration(
        Kind::Wire,
        "q",
        unsigned_width(C::Q::bits()),
        None,
    ));

    // Next, for each sub-component, we need to determine it's input range from the Q and D types.
    // Loop over the components.
    let component_decls = descriptor
        .children
        .iter()
        .enumerate()
        .map(|(ndx, (local, desc))| {
            component_decl::<C>(ndx, local, desc).map(|x| Statement::ComponentInstance(x))
        })
        .collect::<Result<Vec<_>>>()?;
    let design = compile_design::<C::Update>(crate::CompilationMode::Asynchronous)?;
    let verilog = generate_verilog(&design)?;
    let fn_call = continuous_assignment("od", function_call(&verilog.name, vec![id("i"), id("q")]));
    let fn_body = &verilog.body;
    let o_bind = continuous_assignment("o", index_range(id("od"), 0..outputs));
    let d_bind = continuous_assignment("d", index_range(id("od"), outputs..o_d_bits));
    let code = format!(
        "{module_decl}
{od_decl}
{d_decl}
{q_decl}
{o_bind}
{d_bind}

{component_decls}

{fn_call}

{fn_body}

endmodule

",
    );
    Ok(HDLDescriptor {
        name: module_name.into(),
        body: code,
        children: Default::default(),
    })
}

fn component_decl<C: Circuit>(
    ndx: usize,
    local_name: &str,
    desc: &CircuitDescriptor,
) -> Result<ComponentInstance> {
    // instantiate the component with name components.name.
    // give it a unique instance name of c{ndx}
    // wire the inputs to the range of d that corresponds to the name
    // wire the outputs to the range of q that corresponds to the name
    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();
    let (d_range, _) = bit_range(d_kind, &Path::default().field(local_name))?;
    let (q_range, _) = bit_range(q_kind, &Path::default().field(local_name))?;
    let mut instance = ComponentInstance::default();
    instance.name = desc.unique_name.clone();
    instance.instance_name = format!("c{}", ndx);
    let d = index_range(id("d"), d_range);
    let q = index_range(id("q"), q_range);
    instance.connections.push(connection("i", d));
    instance.connections.push(connection("o", q));
    Ok(instance)
}
