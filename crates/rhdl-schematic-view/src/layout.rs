use std::collections::HashMap;

use egui::{pos2, vec2};
use rhdl_core::circuit::schematic::{PortId, Schematic, SchematicSet};

use crate::shape_editor::{Drawing, GRID_SIZE, LabelSide, LineAnchor};

// Create a drawing that represents a thumbnail view of a schematic
pub fn add_thumbnail(schematic: &Schematic, drawing: &mut Drawing) -> HashMap<PortId, LineAnchor> {
    let num_inputs = schematic.inputs.iter().flatten().count();
    let num_outputs = schematic.outputs.len();
    let height = (num_inputs.max(num_outputs) as f32 + 1.0) * GRID_SIZE;
    let width = 10.0 * GRID_SIZE;
    let outer = drawing.add_rect_box(pos2(100.0, 100.0), vec2(width, height));
    let mid_height = height / 2.0;
    let mut port_map = schematic
        .inputs
        .iter()
        .flatten()
        .enumerate()
        .map(|(i, input)| {
            let line_anchor = outer.add_label(
                format!("i{:?}", input.path),
                LabelSide::West,
                GRID_SIZE * (i as f32 + 1.0) - mid_height,
            );
            eprintln!(
                "Input port: {:?} -> line anchor: {:?}",
                input.id, line_anchor
            );
            (input.id, line_anchor)
        })
        .collect::<HashMap<PortId, LineAnchor>>();
    port_map.extend(schematic.outputs.iter().enumerate().map(|(i, output)| {
        let line_anchor = outer.add_label(
            format!("o{:?}", output.path),
            LabelSide::East,
            GRID_SIZE * (i as f32 + 1.0) - mid_height,
        );
        eprintln!(
            "Output port: {:?} -> line anchor: {:?}",
            output.id, line_anchor
        );
        (output.id, line_anchor)
    }));
    port_map
}

// Create a drawing that represents the internal view of a schematic
pub fn add_detail(set: &SchematicSet, drawing: &mut Drawing) {
    let schematic = &set.schematics;
    // For each input port, create a single item off to the left of the drawing
    let mut port_map = schematic
        .inputs
        .iter()
        .flatten()
        .enumerate()
        .map(|(ndx, port)| {
            let item = drawing.add_rect_box(
                pos2(-100.0, 100.0 + ndx as f32 * GRID_SIZE),
                vec2(50.0, 20.0),
            );
            let anchor = item.add_label(format!("i{:?}", port.path), LabelSide::East, 0.0);
            (port.id, anchor)
        })
        .collect::<HashMap<PortId, LineAnchor>>();
    port_map.extend(schematic.outputs.iter().enumerate().map(|(index, port)| {
        let item = drawing.add_rect_box(
            pos2(800.0, 100.0 + index as f32 * GRID_SIZE),
            vec2(50.0, 20.0),
        );
        let anchor = item.add_label(format!("o{:?}", port.path), LabelSide::West, 0.0);
        (port.id, anchor)
    }));

    // For each output port, create a single item off to the right of the drawing
    for (i, output) in schematic.outputs.iter().enumerate() {
        let item =
            drawing.add_rect_box(pos2(800.0, 100.0 + i as f32 * GRID_SIZE), vec2(50.0, 20.0));
        item.add_label(format!("o{:?}", output.path), LabelSide::West, 0.0);
    }

    // for each child schematic, add it's thumbnail to the drawing.
    for child in &schematic.inner {
        port_map.extend(add_thumbnail(child, drawing));
    }
    for pair in set.loop_ports.windows(2) {
        let from_port = schematic.find_port_by_id(pair[0]).unwrap();
        let to_port = schematic.find_port_by_id(pair[1]).unwrap();
        eprintln!("Loop pair: {from_port:?} -> {to_port:?}");
        let Some(from) = port_map.get(&pair[0]) else {
            continue;
        };
        let Some(to) = port_map.get(&pair[1]) else {
            continue;
        };
        eprintln!(" ** Found line anchors: {:?} -> {:?}", from, to);
        drawing.add_line(*from, &[], *to);
    }
}
