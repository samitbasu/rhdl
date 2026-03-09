//! A debug output of a Schematic to a DOT file.
//! The rules for generating the DOT are:
//!
//! - Each input is mapped to a Record node, with one row per port.
//! - The output is mapped to a Record node, with one row per port.
//! - Each internal schematic is mapped to a record node with
//!    - The inputs on the left as a set of rows
//!    - The name of the schematic as the center of the record
//!    - The outputs on the right as a set of rows
//! - Each link is mapped as a directed edge from the source port to the destination port

use super::{Port, PortId, Schematic};
use std::collections::HashMap;

pub fn to_dot(schematic: &Schematic) -> String {
    let mut output = String::new();
    output.push_str("digraph {\n");
    output.push_str("  rankdir=LR;\n");
    output.push_str("  node [shape=record];\n");

    // Map port IDs to their node names and port labels
    let mut port_map: HashMap<PortId, String> = HashMap::new();

    // Generate the inputs node
    if !schematic.inputs.is_empty() {
        let input_label = generate_input_label(&schematic.inputs, &mut port_map);
        output.push_str(&format!("  inputs [label=\"{}\"];\n", input_label));
    }

    // Generate the outputs node
    if !schematic.outputs.is_empty() {
        let output_label = generate_output_label(&schematic.outputs, &mut port_map);
        output.push_str(&format!("  outputs [label=\"{}\"];\n", output_label));
    }

    // Generate nodes for each inner schematic
    for (idx, inner) in schematic.inner.iter().enumerate() {
        let node_name = format!("inner_{}", idx);
        let label = generate_schematic_label(inner, &node_name, &mut port_map);
        output.push_str(&format!("  {} [label=\"{}\"];\n", node_name, label));
    }

    // Generate edges for all links
    for link in &schematic.links {
        if let (Some(from_node), Some(to_node)) = (port_map.get(&link.from), port_map.get(&link.to))
        {
            output.push_str(&format!("  {} -> {};\n", from_node, to_node));
        }
    }

    output.push_str("}\n");
    output
}

fn generate_input_label(inputs: &[Vec<Port>], port_map: &mut HashMap<PortId, String>) -> String {
    let mut parts = Vec::new();
    for group in inputs.iter() {
        for port in group {
            let port_label = format!("p{}", port.id.0);
            let node_ref = format!("inputs:{}", port_label);
            port_map.insert(port.id, node_ref);
            parts.push(format!(
                "<{}> {} ({}b)",
                port_label,
                escape_label(&format!("{:?}", port.path)),
                port.bits
            ));
        }
    }
    parts.join(" | ")
}

fn generate_output_label(outputs: &[Port], port_map: &mut HashMap<PortId, String>) -> String {
    let mut parts = Vec::new();
    for port in outputs {
        let port_label = format!("p{}", port.id.0);
        let node_ref = format!("outputs:{}", port_label);
        port_map.insert(port.id, node_ref);
        parts.push(format!(
            "<{}> {} ({}b)",
            port_label,
            escape_label(&format!("{:?}", port.path)),
            port.bits
        ));
    }
    parts.join(" | ")
}

fn generate_schematic_label(
    schematic: &Schematic,
    node_name: &str,
    port_map: &mut HashMap<PortId, String>,
) -> String {
    // Left side: inputs (vertical stack)
    let mut input_parts = Vec::new();
    for group in &schematic.inputs {
        for port in group {
            let port_label = format!("p{}", port.id.0);
            let node_ref = format!("{}:{}", node_name, port_label);
            port_map.insert(port.id, node_ref);
            input_parts.push(format!(
                "<{}> {} ({}b)",
                port_label,
                escape_label(&format!("{:?}", port.path)),
                port.bits
            ));
        }
    }

    // Center: name
    let name_str = escape_label(&schematic.name);

    // Right side: outputs (vertical stack)
    let mut output_parts = Vec::new();
    for port in &schematic.outputs {
        let port_label = format!("p{}", port.id.0);
        let node_ref = format!("{}:{}", node_name, port_label);
        port_map.insert(port.id, node_ref);
        output_parts.push(format!(
            "<{}> {} ({}b)",
            port_label,
            escape_label(&format!("{:?}", port.path)),
            port.bits
        ));
    }

    // Create horizontal record: {inputs} | name | {outputs}
    // where inputs and outputs are vertical stacks
    let mut label_components = Vec::new();

    if !input_parts.is_empty() {
        label_components.push(format!("{{ {} }}", input_parts.join(" | ")));
    }

    label_components.push(name_str);

    if !output_parts.is_empty() {
        label_components.push(format!("{{ {} }}", output_parts.join(" | ")));
    }

    format!("{{{}}}", label_components.join(" | "))
}

fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('|', "\\|")
}
