use clap::Parser;
use eframe::Result;
use egui::{Color32, Scene, Ui};
use rhdl_core::circuit::schematic::{Port, PortId, Schematic, SchematicId, SchematicKind};
use std::{collections::HashSet, path::PathBuf};
use taffy::{TaffyResult, prelude::*};

const COLORS: [Color32; 7] = [
    Color32::LIGHT_BLUE,
    Color32::LIGHT_GREEN,
    Color32::LIGHT_YELLOW,
    Color32::LIGHT_RED,
    Color32::LIGHT_GRAY,
    Color32::MAGENTA,
    Color32::DARK_GRAY,
];

/// The basic element of the schematic - a rounded box
/// Placement of the element is determined by Taffy.
struct Box {
    label: String,
    help_text: String,
    colors: Color32,
    rounding: f32,
    sid: Option<SchematicId>,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    schematic: PathBuf,
}

pub(crate) struct Rendering {
    taffy_tree: TaffyTree<Box>,
    highlighted_ports: HashSet<PortId>,
    options: Options,
}

impl Default for Rendering {
    fn default() -> Self {
        Self {
            taffy_tree: TaffyTree::new(),
            highlighted_ports: HashSet::new(),
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
        colors: Color32,
        rounding: f32,
        width: f32,
        height: Option<f32>,
        sid: Option<SchematicId>,
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
                sid,
            },
        )?;
        Ok(node_id)
    }
    fn input_port_stack(
        &mut self,
        colors: Color32,
        ports: &[Port],
        index: usize,
    ) -> TaffyResult<NodeId> {
        let children = ports
            .iter()
            .map(|port| {
                self.add_box(
                    format!("i{}{:?}", index, port.path),
                    format!("{:?}", port),
                    if !self.highlighted_ports.contains(&port.id) {
                        colors
                    } else {
                        Color32::RED
                    },
                    self.options.rounding,
                    self.options.input_block_width,
                    Some(20.0),
                    None,
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
                    if self.highlighted_ports.contains(&port.id) {
                        Color32::RED
                    } else {
                        COLORS[6]
                    },
                    self.options.rounding,
                    self.options.input_block_width,
                    Some(20.0),
                    None,
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
        top_schematic_id: &mut SchematicId,
        node_id: NodeId,
        parent_x: f32,
        parent_y: f32,
        ui: &mut Ui,
    ) -> TaffyResult<()> {
        let layout = self.taffy_tree.layout(node_id)?;
        let abs_x = parent_x + layout.location.x;
        let abs_y = parent_y + layout.location.y;
        if let Some(context) = self.taffy_tree.get_node_context(node_id) {
            let rect = egui::Rect::from_min_size(
                egui::pos2(abs_x, abs_y),
                egui::vec2(layout.size.width, layout.size.height),
            );
            ui.painter()
                .rect_filled(rect, context.rounding, context.colors);
            if let Some(id) = context.sid
                && ui
                    .interact(rect, format!("sid{:?}", id).into(), egui::Sense::click())
                    .double_clicked()
            {
                println!("Double clicked on schematic with id: {:?}", id);
                *top_schematic_id = id;
            }
            ui.painter().rect_stroke(
                rect,
                context.rounding,
                (1.0, Color32::BLACK),
                egui::StrokeKind::Middle,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &context.label,
                egui::FontId::monospace(9.0),
                Color32::BLACK,
            );
        }
        // Recursively render children
        for child_id in self.taffy_tree.children(node_id)? {
            self.render_node_recursive(top_schematic_id, child_id, abs_x, abs_y, ui)?;
        }
        Ok(())
    }

    pub(crate) fn render(
        mut self,
        top_schematic_id: &mut SchematicId,
        top: NodeId,
        ui: &mut Ui,
    ) -> TaffyResult<()> {
        // Create an SVG document that contains the rendered Taffy tree
        self.taffy_tree.compute_layout(top, Size::MAX_CONTENT)?;
        let layout = self.taffy_tree.layout(top)?;
        let rect = egui::Rect::from_min_size(
            egui::pos2(layout.location.x, layout.location.y),
            egui::vec2(layout.size.width, layout.size.height),
        );
        ui.painter().rect_filled(rect, 0.0, Color32::DARK_GRAY);
        if ui
            .interact(rect, "schematic".into(), egui::Sense::click())
            .double_clicked()
        {
            println!("Double clicked on background");
            *top_schematic_id = (!0).into();
        }
        self.render_node_recursive(top_schematic_id, top, 0.0, 0.0, ui)
    }

    fn highlight_ports(&mut self, port_loop: &[PortId]) {
        self.highlighted_ports.clear();
        self.highlighted_ports.extend(port_loop.iter().copied());
    }
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
    sid: SchematicId,
) -> TaffyResult<NodeId> {
    let input_ports = render.input_ports(input_ports).unwrap();
    let body = render.add_box(
        label.to_string(),
        help.to_string(),
        COLORS[5],
        render.options.rounding,
        300.0,
        None,
        Some(sid),
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
    let direction = match schematic.kind {
        SchematicKind::Kernel | SchematicKind::OpCode | SchematicKind::Literal => {
            FlexDirection::Row
        }
        _ => FlexDirection::RowReverse,
    };
    render_ports_to_node(
        render,
        &schematic.inputs,
        &schematic.outputs,
        &schematic.name,
        &format!("{:?} {:?}", schematic.kind, schematic.name),
        direction,
        schematic.id,
    )
}

fn find_schematic_by_id(schematic: &Schematic, id: SchematicId) -> Option<&Schematic> {
    if schematic.id == id {
        return Some(schematic);
    }
    for child in &schematic.inner {
        if let Some(found) = find_schematic_by_id(child, id) {
            return Some(found);
        }
    }
    None
}

fn layout_tee(
    top_schematic_id: &mut SchematicId,
    schematic: &Schematic,
    port_loop: &[PortId],
    ui: &mut Ui,
) -> TaffyResult<()> {
    let mut render = Rendering::default();
    render.highlight_ports(port_loop);
    let schematic = find_schematic_by_id(schematic, *top_schematic_id).unwrap();
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
    render.render(top_schematic_id, top, ui)
}

fn main() -> Result {
    let args = Args::parse();

    let content =
        std::boxed::Box::new(std::fs::read_to_string(args.schematic).expect("Failed to read file"));
    let (schematic, port_loop) =
        serde_json::from_str::<(Schematic, Vec<PortId>)>(content.leak()).unwrap();
    let mut scene_rect = egui::Rect::ZERO;
    let options = eframe::NativeOptions::default();
    let mut schematic_stack_id = vec![schematic.id];
    let mut top_schematic_id = schematic.id;
    eframe::run_simple_native("Schematic View", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let scene = Scene::new().zoom_range(0.1..=2.0);
            scene.show(ui, &mut scene_rect, |ui| {
                let prev_top_schematic_id = top_schematic_id;
                if ui.button("Go Back").clicked() && schematic_stack_id.len() > 1 {
                    schematic_stack_id.pop();
                    top_schematic_id = *schematic_stack_id.last().unwrap();
                    println!(
                        "Navigated back to schematic with id: {:?}",
                        top_schematic_id
                    );
                }
                let _ = layout_tee(&mut top_schematic_id, &schematic, &port_loop, ui);
                if top_schematic_id == (!0).into() {
                    if schematic_stack_id.len() > 1 {
                        schematic_stack_id.pop();
                    }
                    top_schematic_id = *schematic_stack_id.last().unwrap();
                } else if prev_top_schematic_id != top_schematic_id {
                    println!("Navigated to schematic with id: {:?}", top_schematic_id);
                    schematic_stack_id.push(top_schematic_id);
                }
            });
        });
    })
}
