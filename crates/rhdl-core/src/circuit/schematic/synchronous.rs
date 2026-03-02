use std::sync::Arc;

use crate::{
    ClockReset, Descriptor, RHDLError, ScopedName, SyncKind, Synchronous, SynchronousIO,
    circuit::schematic::{
        CanonicalPath, Schematic,
        builder::{QueryPortSet, SchematicBuilder},
    },
    rhif,
    types::digital::Digital,
};

pub fn build_schematic<C: Synchronous>(
    scoped_name: &ScopedName,
    kernel: Arc<rhif::Object>,
    children: &[Descriptor<SyncKind>],
) -> Result<Schematic, RHDLError> {
    let circuit_input = <C as SynchronousIO>::I::static_kind();
    let circuit_output = <C as SynchronousIO>::O::static_kind();
    let cr_kind = ClockReset::static_kind();
    let mut builder = SchematicBuilder::synchronous::<C>(&scoped_name.to_string())?;
    let kernel_index = builder.import(super::kernel::build_schematic(kernel)?);
    // Connect the clock and reset signal to the kernel arg 0
    builder.select_and_link(
        QueryPortSet::Input { index: 0 },
        QueryPortSet::ChildInput {
            child_index: kernel_index,
            index: 0,
        },
        cr_kind,
        &CanonicalPath::default(),
        &CanonicalPath::default(),
    )?;
    // Connect the circuit input to the kernel arg 1
    builder.select_and_link(
        QueryPortSet::Input { index: 1 },
        QueryPortSet::ChildInput {
            child_index: kernel_index,
            index: 1,
        },
        circuit_input,
        &CanonicalPath::default(),
        &CanonicalPath::default(),
    )?;
    // Connect the kernel output.0 to the circuit output
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
        builder.select_and_link(
            QueryPortSet::Input { index: 0 },
            QueryPortSet::ChildInput {
                child_index,
                index: 0,
            },
            cr_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::ChildOutput {
                child_index: kernel_index,
            },
            QueryPortSet::ChildInput {
                child_index,
                index: 1,
            },
            child.input_kind,
            &CanonicalPath::default().tuple_index(1).field(local_name),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::ChildOutput { child_index },
            QueryPortSet::ChildInput {
                child_index: kernel_index,
                index: 2,
            },
            child.output_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default().field(local_name),
        )?;
    }
    let schematic = builder.build();
    schematic.checked()?;
    Ok(schematic)
}
