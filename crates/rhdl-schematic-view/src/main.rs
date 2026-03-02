use clap::Parser;
use eframe::Result;
use egui::{Color32, Scene, Ui, Widget};
use rhdl_core::{
    circuit::schematic::{Port, PortId, Schematic, SchematicId, SchematicKind, SchematicSet, Slot},
    trace::container::svg::options,
};
use std::{collections::HashSet, path::PathBuf};
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

#[derive(Default)]
struct KernelDetails {
    name: String,
    filename: String,
    table: Vec<KernelTableEntry>,
}

impl egui::Widget for &KernelDetails {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let font = egui::FontId::monospace(10.0);
        egui::Frame::new()
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(&format!("Name: {}", self.name)).font(font.clone()));
                ui.label(
                    egui::RichText::new(&format!("Filename: {}", self.filename)).font(font.clone()),
                );
                egui::ScrollArea::both().show(ui, |ui| {
                    egui::Grid::new("code")
                        .num_columns(4)
                        .striped(true)
                        .min_row_height(12.0)
                        .show(ui, |ui| {
                            for entry in &self.table {
                                ui.label(egui::RichText::new(&entry.inputs).font(font.clone()));
                                let opcode_cut = &entry.opcode[..(entry.opcode.len().min(25))];
                                ui.label(egui::RichText::new(opcode_cut).font(font.clone()))
                                    .on_hover_text(
                                        egui::RichText::new(&entry.opcode).font(font.clone()),
                                    );
                                ui.label(egui::RichText::new(&entry.outputs).font(font.clone()));
                                ui.label(
                                    egui::RichText::new(&entry.source_code).font(font.clone()),
                                );
                                ui.end_row();
                            }
                        });
                });
            });
        ui.response()
    }
}

struct KernelTableEntry {
    inputs: String,
    opcode: String,
    outputs: String,
    source_code: String,
}

/// The basic element of the schematic - a rounded box
/// Placement of the element is determined by Taffy.
struct Element {
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

fn slot(slot: &Option<Slot>) -> String {
    slot.as_ref()
        .map(|s| match s {
            Slot::Literal(ndx) => format!("l{ndx}"),
            Slot::Register(ndx) => format!("r{ndx}"),
        })
        .unwrap_or_default()
}

fn render_kernel(schematic: &Schematic, highlighted_ports: &HashSet<PortId>) -> KernelDetails {
    if schematic.kind != SchematicKind::Kernel {
        return KernelDetails::default();
    }
    let mut result = Vec::new();
    // Add a table entry for each argument
    for (index, portset) in schematic.inputs.iter().enumerate() {
        for input_port in portset.iter() {
            if highlighted_ports.contains(&input_port.id) {
                result.push(KernelTableEntry {
                    inputs: "Input".to_string(),
                    opcode: format!("i{index}{:?}", input_port.path),
                    outputs: slot(&input_port.slot),
                    source_code: String::new(),
                });
            }
        }
    }
    for child in &schematic.inner {
        match child.kind {
            SchematicKind::OpCode => {
                if !has_highlighted_ports(child, highlighted_ports) {
                    continue;
                }
            }
            SchematicKind::Circuit | SchematicKind::Synchronous | SchematicKind::Kernel => {
                continue;
            }
            _ => {}
        }
        result.push(KernelTableEntry {
            inputs: child
                .inputs
                .iter()
                .flatten()
                .filter(|x| highlighted_ports.contains(&x.id))
                .map(|port| format!("{}{:?}", slot(&port.slot), port.path))
                .collect::<Vec<_>>()
                .join(", "),
            opcode: child.name.clone(),
            outputs: child
                .outputs
                .iter()
                .filter(|x| highlighted_ports.contains(&x.id))
                .map(|port| format!("{}{:?}", slot(&port.slot), port.path))
                .collect::<Vec<_>>()
                .join(", "),
            source_code: String::new(),
        });
    }
    for output_port in &schematic.outputs {
        if highlighted_ports.contains(&output_port.id) {
            result.push(KernelTableEntry {
                inputs: "Output".to_string(),
                opcode: format!("o{:?}", output_port.path),
                outputs: slot(&output_port.slot),
                source_code: String::new(),
            });
        }
    }
    KernelDetails {
        name: schematic.name.clone(),
        filename: schematic.filename.clone(),
        table: result,
    }
}

struct SchematicRendering<'a> {
    options: Options,
    taffy_tree: TaffyTree<Element>,
    highlighted_ports: &'a HashSet<PortId>,
}

fn has_highlighted_ports(schematic: &Schematic, highlighted_ports: &HashSet<PortId>) -> bool {
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
    fn new(options: Options, highlighted_ports: &'a HashSet<PortId>) -> Self {
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
    fn add_portless_box(&mut self, schematic: &Schematic) -> TaffyResult<NodeId> {
        let lines = schematic.name.lines().count().max(1) as f32;
        self.add_leaf_element(
            Element {
                label: schematic.name.to_string(),
                help_text: schematic.type_name.to_string(),
                colors: COLORS[5],
                rounding: self.options.rounding,
                sid: Some(schematic.id),
            },
            500.0,
            Some(lines * 20.0),
        )
    }
    fn layout_kernel(mut self, schematic: &Schematic) -> TaffyResult<(NodeId, TaffyTree<Element>)> {
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
    fn layout_tee(mut self, schematic: &Schematic) -> TaffyResult<(NodeId, TaffyTree<Element>)> {
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

enum Pane {
    Schematic,
    Code,
}

struct App {
    schematic: Schematic,
    highlighted_ports: HashSet<PortId>,
    id_stack: Vec<SchematicId>,
    scene_rect: egui::Rect,
    taffy_tree: TaffyTree<Element>,
    top: NodeId,
    options: Options,
    viz_tree: Option<egui_tiles::Tree<Pane>>,
    update_target: Option<SchematicId>,
    kernel: Option<KernelDetails>,
}

impl egui_tiles::Behavior<Pane> for App {
    fn pane_ui(
        &mut self,
        ui: &mut Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Schematic => {
                self.draw_schematic(ui);
            }
            Pane::Code => {
                self.kernel.as_ref().map(|kernel| {
                    kernel.ui(ui);
                });
            }
        }
        egui_tiles::UiResponse::default()
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Schematic => "Schematic".into(),
            Pane::Code => "Code".into(),
        }
    }
}

impl App {
    fn new(schematic: Schematic, highlighted_ports: HashSet<PortId>) -> Self {
        let options = Options::default();
        let (top, taffy_tree) = SchematicRendering::new(options, &highlighted_ports)
            .layout_tee(&schematic)
            .unwrap();
        let tree = egui_tiles::Tree::new_horizontal("my_tree", vec![Pane::Schematic, Pane::Code]);
        Self {
            id_stack: vec![schematic.id],
            schematic,
            highlighted_ports,
            scene_rect: egui::Rect::ZERO,
            taffy_tree,
            top,
            options,
            viz_tree: Some(tree),
            update_target: None,
            kernel: None,
        }
    }
    fn go_up(&mut self) {
        if self.id_stack.len() > 1 {
            self.id_stack.pop();
        }
    }
    fn top_schematic_id(&self) -> SchematicId {
        *self.id_stack.last().unwrap()
    }
    fn update_render(&mut self) {
        let top_id = self.top_schematic_id();
        let schematic = self.schematic.find_by_id(top_id).unwrap();
        let options = self.options;
        let highlighted_ports = &self.highlighted_ports;
        let renderer = SchematicRendering::new(options, highlighted_ports);
        let (top, taffy_tree) = if matches!(schematic.kind, SchematicKind::Kernel) {
            renderer.layout_kernel(schematic)
        } else {
            renderer.layout_tee(schematic)
        }
        .unwrap();
        self.taffy_tree = taffy_tree;
        self.top = top;
        if let Some(first) = schematic.inner.first()
            && SchematicKind::Kernel == first.kind
        {
            self.kernel = Some(render_kernel(first, highlighted_ports));
        } else {
            self.kernel = None;
        }
    }
    fn draw_node_recursive(
        &mut self,
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
                    .interact(
                        rect,
                        format!("sid{:?}", context.sid).into(),
                        egui::Sense::click(),
                    )
                    .double_clicked()
            {
                self.update_target = Some(id);
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
            self.draw_node_recursive(child_id, abs_x, abs_y, ui)?;
        }
        Ok(())
    }
    fn draw_tree(&mut self, ui: &mut Ui) {
        let layout = self.taffy_tree.layout(self.top).unwrap();
        let rect = egui::Rect::from_min_size(
            egui::pos2(layout.location.x, layout.location.y),
            egui::vec2(layout.size.width, layout.size.height),
        );
        ui.painter().rect_filled(rect, 0.0, Color32::DARK_GRAY);
        if ui
            .interact(rect, "schematic".into(), egui::Sense::click())
            .double_clicked()
        {
            self.go_up();
            self.update_render();
        }
        self.draw_node_recursive(self.top, 0.0, 0.0, ui).unwrap();
    }
    fn draw_schematic(&mut self, ui: &mut Ui) {
        let scene = Scene::new().zoom_range(0.1..=2.0);
        let mut scene_rect = self.scene_rect;
        scene.show(ui, &mut scene_rect, |ui| {
            self.draw_tree(ui);
        });
        self.scene_rect = scene_rect;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viz_tree = std::mem::take(&mut self.viz_tree).unwrap();
            viz_tree.ui(self, ui);
            self.viz_tree = Some(viz_tree);
            if let Some(target) = self.update_target.take() {
                self.id_stack.push(target);
                self.update_render();
            }
        });
    }
}

fn main() -> Result {
    let args = Args::parse();
    let content =
        std::boxed::Box::new(std::fs::read_to_string(args.schematic).expect("Failed to read file"));
    let set = serde_json::from_str::<SchematicSet>(content.leak()).unwrap();
    let app = App::new(set.schematics, set.loop_ports.into_iter().collect());
    eframe::run_native(
        "Schematic View",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
