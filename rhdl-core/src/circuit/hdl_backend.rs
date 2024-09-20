use std::collections::BTreeMap;

use crate::Digital;
use crate::Tristate;
use crate::{Circuit, HDLDescriptor, RHDLError, Synchronous};
enum Direction {
    Input,
    Output,
    Inout,
}

fn maybe_decl(dir: Option<Direction>, num_bits: usize, name: &str) -> String {
    let dir = match dir {
        Some(Direction::Input) => "input",
        Some(Direction::Output) => "output",
        Some(Direction::Inout) => "inout",
        None => "",
    };
    if num_bits == 0 {
        return String::new();
    }
    format!(
        "{dir} wire[{NUM_BITS}:0] {NAME}",
        dir = dir,
        NUM_BITS = num_bits.saturating_sub(1),
        NAME = name
    )
}

pub fn build_hdl<C: Circuit>(
    circuit: &C,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    let descriptor = circuit.descriptor()?;
    let input_bits = C::I::bits();
    let outputs = C::O::bits();

    let module_name = &descriptor.unique_name;
    let io_decl = maybe_decl(Some(Direction::Inout), C::Z::N, "io");
    let in_decl = maybe_decl(Some(Direction::Input), C::I::bits(), "i");
    let out_decl = maybe_decl(Some(Direction::Output), C::O::bits(), "o");

    let module_decl = format!(
        "module {module_name}({decls});",
        module_name = module_name,
        decls = vec![io_decl, in_decl, out_decl].join(", ")
    );

    let od_decls = maybe_decl(None, C::O::bits() + C::D::bits(), "od");

    let input_decls = if input_bits != 0 {
        format!(
            "input wire[{INPUT_BITS}:0] i",
            INPUT_BITS = input_bits.saturating_sub(1)
        )
    } else {
        Default::default()
    };
}

pub fn build_synchronous_hdl<S: Synchronous>(
    synchronous: &S,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    todo!()
}
