use std::collections::HashSet;

use crate::{
    RHDLError,
    circuit::schematic::{PortId, Schematic, error::SchematicICE},
    error::rhdl_error,
};

pub fn check_connected(schematic: &Schematic) -> Result<(), RHDLError> {
    let mut sinks = HashSet::new();
    collect_sinks(&mut sinks, schematic);
    schematic
        .inner
        .iter()
        .try_for_each(|child| check_inputs_connected(&sinks, child))?;
    Ok(())
}

fn collect_sinks(sinks: &mut HashSet<PortId>, schematic: &Schematic) {
    for link in &schematic.links {
        sinks.insert(link.to);
    }
    for child in &schematic.inner {
        collect_sinks(sinks, child);
    }
}

fn check_inputs_connected(sinks: &HashSet<PortId>, schematic: &Schematic) -> Result<(), RHDLError> {
    for input in schematic.inputs.iter().flatten() {
        if !sinks.contains(&input.id) {
            return Err(rhdl_error(SchematicICE::UnconnectedInputPort {
                block_name: schematic.name.clone(),
                port: input.clone(),
            }));
        }
    }
    for child in &schematic.inner {
        check_inputs_connected(sinks, child)?;
    }
    Ok(())
}
