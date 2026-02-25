use std::collections::HashSet;

use crate::{
    RHDLError,
    circuit::schematic::{PortId, Schematic, SchematicId, error::SchematicICE},
    error::rhdl_error,
};

fn collect_ids(
    schematic: &Schematic,
    port_ids: &mut HashSet<PortId>,
    schematic_ids: &mut HashSet<SchematicId>,
) -> Result<(), RHDLError> {
    for input in schematic.inputs.iter().flatten() {
        if !port_ids.insert(input.id) {
            return Err(rhdl_error(SchematicICE::DuplicatePortId {
                id: input.id,
                sid: schematic.id,
            }));
        }
    }
    for output in &schematic.outputs {
        if !port_ids.insert(output.id) {
            return Err(rhdl_error(SchematicICE::DuplicatePortId {
                id: output.id,
                sid: schematic.id,
            }));
        }
    }
    if !schematic_ids.insert(schematic.id) {
        return Err(rhdl_error(SchematicICE::DuplicateSchematicId {
            id: schematic.id,
        }));
    }
    for child in &schematic.inner {
        collect_ids(child, port_ids, schematic_ids)?;
    }
    Ok(())
}

pub fn check_ids(schematic: &Schematic) -> Result<(), RHDLError> {
    let mut port_ids = HashSet::new();
    let mut schematic_ids = HashSet::new();
    collect_ids(schematic, &mut port_ids, &mut schematic_ids)?;
    Ok(())
}
