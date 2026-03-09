use std::collections::HashSet;

use egui::Color32;
use rhdl_core::circuit::schematic::{Port, PortId, Schematic, SchematicId, SchematicKind};
use taffy::{TaffyResult, prelude::*};

const COLORS: [Color32; 8] = [
    Color32::LIGHT_BLUE,
    Color32::LIGHT_GREEN,
    Color32::LIGHT_YELLOW,
    Color32::LIGHT_RED,
    Color32::LIGHT_GRAY,
    Color32::DARK_GREEN,
    Color32::KHAKI,
    Color32::DARK_GRAY,
];

#[derive(Clone, Copy)]
pub struct Options {
    pub input_block_width: f32,
    pub rounding: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            input_block_width: 200.0,
            rounding: 5.0,
        }
    }
}

/// The basic element of the schematic - a rounded box
/// Placement of the element is determined by Taffy.
pub struct Element {
    pub label: String,
    pub help_text: String,
    pub colors: Color32,
    pub rounding: f32,
    pub sid: Option<SchematicId>,
}

pub struct SchematicRendering<'a> {
    pub options: Options,
    pub taffy_tree: TaffyTree<Element>,
    pub highlighted_ports: &'a HashSet<PortId>,
}

pub fn has_highlighted_ports(schematic: &Schematic, highlighted_ports: &HashSet<PortId>) -> bool {
    for input in schematic.inputs.iter().flatten() {
        if highlighted_ports.contains(&input.id) {
            return true;
        }
    }
    for output in &schematic.outputs {
        if highlighted_ports.contains(&output.id) {
            return true;
        }
    }
    false
}

impl<'a> SchematicRendering<'a> {
    pub fn new(options: Options, highlighted_ports: &'a HashSet<PortId>) -> Self {
        Self {
            options,
            taffy_tree: TaffyTree::new(),
            highlighted_ports,
        }
    }
    fn add_leaf_element(
        &mut self,
        context: Element,
        width: f32,
        height: Option<f32>,
    ) -> TaffyResult<NodeId> {
        let height = height
            .map(Dimension::length)
            .unwrap_or_else(Dimension::auto);
        let node_id = self.taffy_tree.new_leaf_with_context(
            Style {
                size: Size {
                    width: Dimension::length(width),
                    height,
                },
                ..Default::default()
            },
            context,
        )?;
        Ok(node_id)
    }
    fn add_port(&mut self, label: &str, help: &str, color: Color32) -> TaffyResult<NodeId> {
        self.add_leaf_element(
            Element {
                label: label.to_string(),
                help_text: help.to_string(),
                colors: color,
                rounding: self.options.rounding,
                sid: None,
            },
            self.options.input_block_width,
            Some(20.0),
        )
    }
    fn add_input_port_stack(
        &mut self,
        colors: Color32,
        ports: &[Port],
        index: usize,
    ) -> TaffyResult<NodeId> {
        let children = ports
            .iter()
            .map(|port| {
                self.add_port(
                    &format!("i{}{:?}", index, port.path),
                    &format!("{:?}", port),
                    if !self.highlighted_ports.contains(&port.id) {
                        colors
                    } else {
                        Color32::RED
                    },
                )
            })
            .collect::<TaffyResult<Vec<_>>>()?;
        // Lay these out in a vertical column
        self.taffy_tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                align_items: Some(AlignItems::Stretch),
                ..Default::default()
            },
            &children,
        )
    }
    fn add_input_ports(&mut self, ports: &[Vec<Port>]) -> TaffyResult<NodeId> {
        let input_stacks = ports
            .iter()
            .enumerate()
            .map(|(index, group)| {
                self.add_input_port_stack(COLORS[index % COLORS.len()], group, index)
            })
            .collect::<TaffyResult<Vec<_>>>()?;
        self.taffy_tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            &input_stacks,
        )
    }
    fn add_output_ports(&mut self, ports: &[Port]) -> TaffyResult<NodeId> {
        let children = ports
            .iter()
            .map(|port| {
                self.add_port(
                    &format!("o{:?}", port.path),
                    &format!("{:?}", port),
                    if self.highlighted_ports.contains(&port.id) {
                        Color32::RED
                    } else {
                        COLORS[6]
                    },
                )
            })
            .collect::<TaffyResult<Vec<_>>>()?;
        // Lay these out in a vertical column
        self.taffy_tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            &children,
        )
    }
    fn add_black_box(&mut self, schematic: &Schematic) -> TaffyResult<NodeId> {
        let direction = match schematic.kind {
            SchematicKind::Kernel | SchematicKind::OpCode | SchematicKind::Literal => {
                FlexDirection::Row
            }
            _ => FlexDirection::RowReverse,
        };
        let input_ports = self.add_input_ports(&schematic.inputs)?;
        let body = self.add_leaf_element(
            Element {
                label: schematic.name.to_string(),
                help_text: schematic.type_name.to_string(),
                colors: COLORS[5],
                rounding: self.options.rounding,
                sid: Some(schematic.id),
            },
            300.0,
            None,
        )?;
        let output_ports = self.add_output_ports(&schematic.outputs)?;
        self.taffy_tree.new_with_children(
            Style {
                flex_direction: direction,
                align_content: Some(AlignContent::Center),
                ..Default::default()
            },
            &[input_ports, body, output_ports],
        )
    }
    pub fn layout_kernel(
        mut self,
        schematic: &Schematic,
    ) -> TaffyResult<(NodeId, TaffyTree<Element>)> {
        let ops = schematic
            .inner
            .iter()
            .filter_map(|child| match child.kind {
                SchematicKind::Literal => Some(self.add_black_box(child)),
                SchematicKind::OpCode => {
                    if has_highlighted_ports(child, self.highlighted_ports) {
                        Some(self.add_black_box(child))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<TaffyResult<Vec<_>>>()?;
        let node_id = self.taffy_tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                gap: Size {
                    width: LengthPercentage::percent(1.0),
                    height: LengthPercentage::length(25.0),
                },
                ..Default::default()
            },
            &ops,
        )?;
        self.taffy_tree.compute_layout(node_id, Size::MAX_CONTENT)?;
        Ok((node_id, self.taffy_tree))
    }
    pub fn layout_tee(
        mut self,
        schematic: &Schematic,
    ) -> TaffyResult<(NodeId, TaffyTree<Element>)> {
        let child_nodes = schematic
            .inner
            .iter()
            .map(|child| self.add_black_box(child))
            .collect::<TaffyResult<Vec<_>>>()?;
        let center_column = self.taffy_tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                align_content: Some(AlignContent::Center),
                gap: Size {
                    width: LengthPercentage::percent(1.0),
                    height: LengthPercentage::length(150.0),
                },
                ..Default::default()
            },
            &child_nodes,
        )?;
        let input_ports = self.add_input_ports(&schematic.inputs)?;
        let output_ports = self.add_output_ports(&schematic.outputs)?;
        let top = self.taffy_tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Row,
                align_content: Some(AlignContent::Center),
                gap: Size {
                    width: LengthPercentage::length(250.0),
                    height: LengthPercentage::percent(1.0),
                },
                ..Default::default()
            },
            &[input_ports, center_column, output_ports],
        )?;
        self.taffy_tree.compute_layout(top, Size::MAX_CONTENT)?;
        Ok((top, self.taffy_tree))
    }
}
