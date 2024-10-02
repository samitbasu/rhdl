use std::collections::BTreeMap;

use crate::compiler::codegen::verilog::generate_verilog;
use crate::types::path::bit_range;
use crate::types::path::Path;
use crate::verilog::ast::component_instance;
use crate::verilog::ast::concatenate;
use crate::verilog::ast::connection;
use crate::verilog::ast::continuous_assignment;
use crate::verilog::ast::function_call;
use crate::verilog::ast::id;
use crate::verilog::ast::index;
use crate::verilog::ast::unsigned_width;
use crate::verilog::ast::Declaration;
use crate::verilog::ast::Direction;
use crate::verilog::ast::Kind;
use crate::verilog::ast::Module;
use crate::verilog::ast::Port;
use crate::verilog::ast::Statement;
use crate::Digital;
use crate::HDLKind;
use crate::Tristate;
use crate::{Circuit, HDLDescriptor, RHDLError, Synchronous};

fn maybe_port_wire(dir: Direction, num_bits: usize, name: &str) -> Option<Port> {
    (num_bits != 0).then(|| Port {
        direction: dir,
        kind: Kind::Wire,
        name: name.into(),
        width: unsigned_width(num_bits),
    })
}

fn maybe_decl_wire(num_bits: usize, name: &str) -> Option<Declaration> {
    (num_bits != 0).then(|| Declaration {
        kind: Kind::Wire,
        name: name.into(),
        width: unsigned_width(num_bits),
        alias: None,
    })
}

pub fn build_hdl<C: Circuit>(
    circuit: &C,
    children: BTreeMap<String, HDLDescriptor>,
    _kind: HDLKind,
) -> Result<HDLDescriptor, RHDLError> {
    let descriptor = circuit.descriptor()?;
    let outputs = C::O::bits();

    let module_name = &descriptor.unique_name;
    let mut module = Module::default();
    module.name = module_name.clone();
    module.ports = [
        maybe_port_wire(Direction::Input, C::I::bits(), "i"),
        maybe_port_wire(Direction::Output, C::O::bits(), "o"),
        maybe_port_wire(Direction::Inout, C::Z::N, "io"),
    ]
    .into_iter()
    .flatten()
    .collect();
    module.declarations.extend(
        [
            maybe_decl_wire(C::O::bits() + C::D::bits(), "od"),
            maybe_decl_wire(C::D::bits(), "d"),
            maybe_decl_wire(C::Q::bits(), "q"),
        ]
        .into_iter()
        .flatten(),
    );
    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();
    let child_decls = descriptor
        .children
        .iter()
        .enumerate()
        .map(|(ndx, (local_name, descriptor))| {
            let child_path = Path::default().field(local_name);
            let (d_range, _) = bit_range(d_kind.clone(), &child_path)?;
            let (q_range, _) = bit_range(q_kind.clone(), &child_path)?;
            let input_binding =
                (!d_range.is_empty()).then(|| connection("i", index(id("d"), d_range.clone())));
            let output_binding =
                (!q_range.is_empty()).then(|| connection("o", index(id("q"), q_range.clone())));
            let component_name = &descriptor.unique_name;
            Ok(component_instance(
                &component_name,
                &format!("c{ndx}"),
                [input_binding, output_binding]
                    .into_iter()
                    .flatten()
                    .collect(),
            ))
        })
        .collect::<Result<Vec<Statement>, RHDLError>>()?;
    let verilog = generate_verilog(descriptor.rtl.as_ref().unwrap())?;
    // Call the verilog function with (i, q), if they exist.
    let i_bind = (C::I::bits() != 0).then(|| id("i"));
    let q_bind = (C::Q::bits() != 0).then(|| id("q"));
    let fn_call = function_call(
        &verilog.name,
        vec![i_bind, q_bind].into_iter().flatten().collect(),
    );
    let fn_call = continuous_assignment("od", fn_call);
    let o_bind = continuous_assignment("o", index(id("od"), 0..outputs));
    let d_bind = (C::D::bits() != 0)
        .then(|| continuous_assignment("d", index(id("od"), outputs..(C::D::bits() + outputs))));
    module.statements.push(o_bind);
    module.statements.extend(child_decls);
    module.statements.push(fn_call);
    if let Some(d_bind) = d_bind {
        module.statements.push(d_bind);
    }
    module.functions.push(verilog);
    Ok(HDLDescriptor {
        name: module_name.into(),
        body: module,
        children,
    })
}

// There is a fair amount of overlap between this function and the previous one.  In principle,
// it should be possible to factor out the common bits and DRY up the code.
pub fn build_synchronous_hdl<C: Synchronous>(
    circuit: &C,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    let descriptor = circuit.descriptor()?;
    let outputs = C::O::bits();

    let module_name = &descriptor.unique_name;
    let mut module = Module::default();
    module.name = module_name.clone();
    module.ports = [
        maybe_port_wire(Direction::Input, 1, "clock"),
        maybe_port_wire(Direction::Input, 1, "reset"),
        maybe_port_wire(Direction::Input, C::I::bits(), "i"),
        maybe_port_wire(Direction::Output, C::O::bits(), "o"),
        maybe_port_wire(Direction::Inout, C::Z::N, "io"),
    ]
    .into_iter()
    .flatten()
    .collect();
    module.declarations.extend(
        [
            maybe_decl_wire(C::O::bits() + C::D::bits(), "od"),
            maybe_decl_wire(C::D::bits(), "d"),
            maybe_decl_wire(C::Q::bits(), "q"),
        ]
        .into_iter()
        .flatten(),
    );
    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();
    let child_decls = descriptor
        .children
        .iter()
        .enumerate()
        .map(|(ndx, (local_name, descriptor))| {
            let child_path = Path::default().field(local_name);
            let (d_range, _) = bit_range(d_kind.clone(), &child_path)?;
            let (q_range, _) = bit_range(q_kind.clone(), &child_path)?;
            let input_binding = if d_range.is_empty() {
                None
            } else {
                Some(connection("i", index(id("d"), d_range.clone())))
            };
            let output_binding = if q_range.is_empty() {
                None
            } else {
                Some(connection("o", index(id("q"), q_range.clone())))
            };
            let component_name = &descriptor.unique_name;
            let clock_binding = Some(connection("clock", id("clock")));
            let reset_binding = Some(connection("reset", id("reset")));
            Ok(component_instance(
                &component_name,
                &format!("c{ndx}"),
                [clock_binding, reset_binding, input_binding, output_binding]
                    .into_iter()
                    .flatten()
                    .collect(),
            ))
        })
        .collect::<Result<Vec<Statement>, RHDLError>>()?;
    let verilog = generate_verilog(descriptor.rtl.as_ref().unwrap())?;
    // Call the verilog function with (reset, i, q), if they exist.
    let clock_reset = Some(concatenate(vec![id("clock"), id("reset")]));
    let i_bind = (C::I::bits() != 0).then(|| id("i"));
    let q_bind = (C::Q::bits() != 0).then(|| id("q"));
    let fn_call = function_call(
        &verilog.name,
        vec![clock_reset, i_bind, q_bind]
            .into_iter()
            .flatten()
            .collect(),
    );
    let fn_call = continuous_assignment("od", fn_call);
    let o_bind = continuous_assignment("o", index(id("od"), 0..outputs));
    let d_bind = (C::D::bits() != 0)
        .then(|| continuous_assignment("d", index(id("od"), outputs..(C::D::bits() + outputs))));
    module.statements.push(o_bind);
    module.statements.extend(child_decls);
    module.statements.push(fn_call);
    if let Some(d_bind) = d_bind {
        module.statements.push(d_bind);
    }
    module.functions.push(verilog);
    Ok(HDLDescriptor {
        name: module_name.into(),
        body: module,
        children,
    })
}
