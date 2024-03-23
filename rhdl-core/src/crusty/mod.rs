pub mod checks;
pub mod downstream;
pub mod index;
pub mod source_pool;
pub mod upstream;
pub mod utils;

/*
fn check_dff_clock_routing_single(is: &IndexedSchematic, index: usize) -> Result<()> {
    // For the given flip flop component, get the corresponding clock pin
    let ComponentKind::DigitalFlipFlop(component) = &is.schematic.components[index].kind else {
        bail!("ICE - component index did not yield expeced DFF")
    };
    let clock_pin = component.clock;

    Ok(())
}

pub fn check_dff_clock_routing(schematic: Schematic) -> Result<()> {
    // First, we ensure the schematic is inlined.
    let is = make_indexed_schematic(schematic);
    // Next, we search through it to find flip flops.
    let flip_flop_list = is
        .schematic
        .components
        .iter()
        .enumerate()
        .filter(|(_, comp)| matches!(comp.kind, ComponentKind::DigitalFlipFlop(_)))
        .collect::<Vec<_>>();
    // Now for each flip flop component, we find the clock pin.

    Ok(())
}
*/
