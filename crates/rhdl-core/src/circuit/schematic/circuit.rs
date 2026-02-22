use std::sync::Arc;

use crate::circuit::schematic::CanonicalPath;
use crate::circuit::schematic::builder::QueryPortSet;
use crate::types::digital::Digital;
use crate::types::path::Path;
use crate::{
    AsyncKind, Circuit, CircuitIO, Descriptor, ScopedName,
    circuit::schematic::{Schematic, builder::SchematicBuilder},
};
use crate::{RHDLError, rhif};

pub fn build_schematic<C: Circuit>(
    scoped_name: &ScopedName,
    kernel: Arc<rhif::Object>,
    children: &[Descriptor<AsyncKind>],
) -> Result<Schematic, RHDLError> {
    let circuit_input = <C as CircuitIO>::I::static_kind();
    let circuit_output = <C as CircuitIO>::O::static_kind();
    let mut builder = SchematicBuilder::circuit::<C>(&scoped_name.to_string())?;
    let kernel_index = builder.import(super::kernel::build_schematic(kernel)?);
    // Connect all of the schematic input ports to the first argument of the kernel
    builder.select_and_link(
        QueryPortSet::Input { index: 0 },
        QueryPortSet::ChildInput {
            child_index: kernel_index,
            index: 0,
        },
        circuit_input,
        &CanonicalPath::default(),
        &CanonicalPath::default(),
    )?;
    // Connect the kernel output.0 to the schematic output port
    builder.select_and_link(
        QueryPortSet::ChildOutput {
            child_index: kernel_index,
        },
        QueryPortSet::Output,
        circuit_output,
        &CanonicalPath::default().tuple_index(0),
        &CanonicalPath::default(),
    )?;
    for child in children {
        let local_name = child.name.last().unwrap();
        let child_index = builder.import(child.schematic()?.clone());
        // Connect all of the kernel output.1.child_name ports to the child input ports
        builder.select_and_link(
            QueryPortSet::ChildOutput {
                child_index: kernel_index,
            },
            QueryPortSet::ChildInput {
                child_index,
                index: 0,
            },
            child.input_kind,
            &CanonicalPath::default().tuple_index(1).field(local_name),
            &CanonicalPath::default(),
        )?;
        // Connect all of the child output ports to the kernel input#1.child_name ports
        builder.select_and_link(
            QueryPortSet::ChildOutput { child_index },
            QueryPortSet::ChildInput {
                child_index: kernel_index,
                index: 1,
            },
            child.output_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default().field(local_name),
        )?;
    }
    Ok(builder.build())
}
