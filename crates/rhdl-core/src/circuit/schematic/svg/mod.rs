use taffy::{TaffyResult, prelude::*};

use crate::{
    circuit::schematic::{Port, Schematic},
    rhif,
};

#[derive(Copy, Clone)]
struct Colors {
    stroke: &'static str,
    fill: &'static str,
}

/// A static set of colors that are pastel rainbow colors, with
/// strong contrast between the stroke (border) and fill (background) colors.
const COLORS: [Colors; 7] = [
    Colors {
        stroke: "black",
        fill: "lightcoral",
    },
    Colors {
        stroke: "black",
        fill: "lightblue",
    },
    Colors {
        stroke: "black",
        fill: "lightgreen",
    },
    Colors {
        stroke: "black",
        fill: "lightyellow",
    },
    Colors {
        stroke: "black",
        fill: "lightpink",
    },
    Colors {
        stroke: "black",
        fill: "lightcyan",
    },
    Colors {
        stroke: "black",
        fill: "lightgray",
    },
];

/// The basic element of the schematic - a rounded box
/// Placement of the element is determined by Taffy.
struct Box {
    label: String,
    help_text: String,
    colors: Colors,
    rounding: f32,
}

pub(crate) struct Rendering {
    taffy_tree: TaffyTree<Box>,
    options: Options,
}

impl Default for Rendering {
    fn default() -> Self {
        Self {
            taffy_tree: TaffyTree::new(),
            options: Options::default(),
        }
    }
}

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

impl Rendering {
    fn add_box(
        &mut self,
        label: String,
        help_text: String,
        colors: Colors,
        rounding: f32,
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
            Box {
                label,
                help_text,
                colors,
                rounding,
            },
        )?;
        Ok(node_id)
    }
    fn input_port_stack(
        &mut self,
        colors: Colors,
        ports: &[Port],
        index: usize,
    ) -> TaffyResult<NodeId> {
        let children = ports
            .iter()
            .map(|port| {
                self.add_box(
                    format!("i{}{:?}", index, port.path),
                    format!("{:?}", port),
                    colors,
                    self.options.rounding,
                    self.options.input_block_width,
                    Some(20.0),
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
    pub(crate) fn input_ports(&mut self, ports: &[Vec<Port>]) -> TaffyResult<NodeId> {
        let input_stacks = ports
            .iter()
            .enumerate()
            .map(|(index, group)| self.input_port_stack(COLORS[index % COLORS.len()], group, index))
            .collect::<TaffyResult<Vec<_>>>()?;
        self.taffy_tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            &input_stacks,
        )
    }
    pub(crate) fn output_ports(&mut self, ports: &[Port]) -> TaffyResult<NodeId> {
        let children = ports
            .iter()
            .map(|port| {
                self.add_box(
                    format!("o{:?}", port.path),
                    format!("{:?}", port),
                    COLORS[6],
                    self.options.rounding,
                    self.options.input_block_width,
                    Some(20.0),
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
    fn render_node_recursive(
        &self,
        node_id: NodeId,
        parent_x: f32,
        parent_y: f32,
        document: svg::Document,
    ) -> TaffyResult<svg::Document> {
        let layout = self.taffy_tree.layout(node_id)?;
        let abs_x = parent_x + layout.location.x;
        let abs_y = parent_y + layout.location.y;

        let mut doc = document;

        if let Some(context) = self.taffy_tree.get_node_context(node_id) {
            let rect = svg::node::element::Rectangle::new()
                .set("x", abs_x)
                .set("y", abs_y)
                .set("width", layout.size.width)
                .set("height", layout.size.height)
                .set("rx", context.rounding)
                .set("ry", context.rounding)
                .set("fill", context.colors.fill)
                .set("stroke", context.colors.stroke);

            let text = svg::node::element::Text::new(&context.label)
                .set("x", abs_x + 5.0)
                .set("y", abs_y + 15.0)
                .set("font-family", "monospace")
                .set("font-size", 8)
                .set("fill", "black");
            let tip = svg::node::element::Title::new(&context.help_text);
            let text = text.add(tip);
            doc = doc.add(rect).add(text);
        }

        // Recursively render children
        for child_id in self.taffy_tree.children(node_id)? {
            doc = self.render_node_recursive(child_id, abs_x, abs_y, doc)?;
        }

        Ok(doc)
    }

    pub(crate) fn render(mut self, top: NodeId) -> TaffyResult<svg::Document> {
        // Create an SVG document that contains the rendered Taffy tree
        self.taffy_tree.compute_layout(top, Size::MAX_CONTENT)?;
        taffy::print_tree(&self.taffy_tree, top);
        let document = svg::Document::new().set(
            "viewBox",
            (
                0,
                0,
                self.taffy_tree.layout(top)?.size.width,
                self.taffy_tree.layout(top)?.size.height,
            ),
        );

        self.render_node_recursive(top, 0.0, 0.0, document)
    }
}

fn render_kernel_to_node(render: &mut Rendering, rhif: &rhif::Object) -> TaffyResult<NodeId> {
    todo!()
}

// We create a thing that has input ports on the left, output ports on the right
// and a label in the middle.
fn render_ports_to_node(
    render: &mut Rendering,
    input_ports: &[Vec<Port>],
    output_ports: &[Port],
    label: &str,
    help: &str,
    flex_direction: FlexDirection,
) -> TaffyResult<NodeId> {
    let input_ports = render.input_ports(input_ports).unwrap();
    let body = render.add_box(
        label.to_string(),
        help.to_string(),
        COLORS[5],
        render.options.rounding,
        300.0,
        None,
    )?;
    let output_ports = render.output_ports(output_ports).unwrap();
    let top = render
        .taffy_tree
        .new_with_children(
            Style {
                flex_direction,
                align_content: Some(AlignContent::Center),
                ..Default::default()
            },
            &[input_ports, body, output_ports],
        )
        .unwrap();
    Ok(top)
}

fn schematic_inner(render: &mut Rendering, schematic: &Schematic) -> TaffyResult<NodeId> {
    render_ports_to_node(
        render,
        &schematic.inputs,
        &schematic.outputs,
        &schematic.name,
        &format!("{:?} {:?}", schematic.kind, schematic.name),
        FlexDirection::Row,
    )
}

pub(crate) fn schematic(schematic: &Schematic) -> TaffyResult<svg::Document> {
    let mut render = Rendering::default();
    let child_nodes = schematic
        .inner
        .iter()
        .map(|child| schematic_inner(&mut render, child))
        .collect::<TaffyResult<Vec<_>>>()?;
    let center_column = render.taffy_tree.new_with_children(
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
    let input_ports = render.input_ports(&schematic.inputs).unwrap();
    let output_ports = render.output_ports(&schematic.outputs).unwrap();
    let top = render
        .taffy_tree
        .new_with_children(
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
        )
        .unwrap();
    render
        .taffy_tree
        .compute_layout(top, Size::MAX_CONTENT)
        .unwrap();
    render.render(top)
}
