use clap::Parser;
use eframe::Result;
use egui::{Align2, Color32, Id, Pos2, Scene, Ui, Vec2, Widget, pos2};
use rhdl_core::circuit::schematic::{
    PortId, PortPosition, Schematic, SchematicId, SchematicKind, SchematicSet, Slot,
};
use std::{collections::HashSet, path::PathBuf};
use taffy::{TaffyResult, prelude::*};

use crate::{
    avoid::grid,
    kernel_details::{KernelDetails, render_kernel},
    schematic_rendering::{Element, Options, SchematicRendering},
};
pub mod avoid;
pub mod kernel_details;
pub mod layout;
pub mod schematic_rendering;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    schematic: PathBuf,
}

fn slot(slot: &Option<Slot>) -> String {
    slot.as_ref()
        .map(|s| match s {
            Slot::Literal(ndx) => format!("l{ndx}"),
            Slot::Register(ndx) => format!("r{ndx}"),
        })
        .unwrap_or_default()
}

enum Pane {
    Schematic,
    Code,
}

enum PortAction {
    Hover,
    Click,
    Drag(Pos2, Vec2),
}

struct App {
    filename: PathBuf,
    schematic: Schematic,
    highlighted_ports: HashSet<PortId>,
    id_stack: Vec<SchematicId>,
    scene_rect: egui::Rect,
    detail_scene_rect: egui::Rect,
    taffy_tree: TaffyTree<Element>,
    top: NodeId,
    options: Options,
    viz_tree: Option<egui_tiles::Tree<Pane>>,
    update_target: Option<SchematicId>,
    kernel: Option<KernelDetails>,
    pin_drag: Option<PortId>,
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
                self.draw_schematic_detail(ui);
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
    fn new(filename: PathBuf) -> Self {
        let content = std::fs::read_to_string(&filename).expect("Failed to read file");
        let set = serde_json::from_str::<SchematicSet>(&content).unwrap();
        let schematic = set.schematics;
        let highlighted_ports = set.loop_ports.into_iter().collect();
        let options = Options::default();
        let (top, taffy_tree) = SchematicRendering::new(options, &highlighted_ports)
            .layout_tee(&schematic)
            .unwrap();
        let tree = egui_tiles::Tree::new_horizontal("my_tree", vec![Pane::Schematic, Pane::Code]);
        Self {
            filename,
            id_stack: vec![schematic.id],
            schematic,
            highlighted_ports,
            scene_rect: egui::Rect::ZERO,
            detail_scene_rect: egui::Rect::ZERO,
            taffy_tree,
            top,
            options,
            viz_tree: Some(tree),
            update_target: None,
            kernel: None,
            pin_drag: None,
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
    fn draw_pin(
        ui: &mut Ui,
        port_id: PortId,
        pin_position: Pos2,
        pin_name: &str,
        align: Align2,
    ) -> Option<PortAction> {
        let mut action = None;
        ui.painter().circle_filled(pin_position, 3.0, Color32::RED);
        // Is the circle hovered?
        let response = ui.interact(
            egui::Rect::from_center_size(pin_position, egui::vec2(10.0, 10.0)),
            Id::new(("port_id", port_id.index())),
            egui::Sense::click_and_drag(),
        );
        if response.dragged() {
            let delta = response.drag_delta();
            ui.painter()
                .circle_stroke(pin_position + delta, 4.0, (1.0, Color32::LIGHT_BLUE));
            action = Some(PortAction::Drag(
                response.interact_pointer_pos().unwrap_or(pin_position),
                delta,
            ));
        } else if response.clicked() {
            action = Some(PortAction::Click);
        } else if response.hovered() {
            ui.painter()
                .circle_stroke(pin_position, 4.0, (1.0, Color32::YELLOW));
            action = Some(PortAction::Hover);
        }
        let delta = if align == Align2::LEFT_CENTER {
            egui::vec2(5.0, 0.0)
        } else {
            egui::vec2(-5.0, 0.0)
        };
        ui.painter().text(
            pin_position + delta,
            align,
            pin_name,
            egui::FontId::monospace(9.0),
            Color32::BLACK,
        );
        action
    }
    fn draw_schematic_plan_view(&mut self, sid: SchematicId, ui: &mut Ui) {
        if let Some(schematic) = self.schematic.find_by_id_mut(sid) {
            let grid = grid(schematic);
            let children = schematic
                .inner
                .iter()
                .map(|child| child.id)
                .collect::<Vec<_>>();
            for child_id in children {
                self.draw_plan_view(child_id, ui);
            }
            for (y_ndx, y_pos) in grid.y.iter().enumerate() {
                for (x_ndx, x_pos) in grid.x.iter().enumerate() {
                    if !grid.occupied[y_ndx][x_ndx] {
                        ui.painter().circle_filled(
                            pos2((*x_pos).into(), (*y_pos).into()),
                            2.0,
                            Color32::LIGHT_GRAY,
                        );
                    }
                }
            }
        }
    }
    fn draw_plan_view(&mut self, sid: SchematicId, ui: &mut Ui) {
        if let Some(schematic) = self.schematic.find_by_id_mut(sid) {
            // Draw a rectangle for the schematic item - rounded.
            let outer_rect = egui::Rect::from_min_size(
                egui::pos2(schematic.origin.0.into(), schematic.origin.1.into()),
                egui::vec2(schematic.size.0.into(), schematic.size.1.into()),
            );
            ui.painter().rect(
                outer_rect,
                3.0,
                Color32::LIGHT_GRAY,
                (1.0, Color32::DARK_BLUE),
                egui::StrokeKind::Middle,
            );
            let response = ui
                .interact(
                    egui::Rect::from_center_size(outer_rect.right_bottom(), egui::vec2(5.0, 5.0)),
                    Id::new(("schematic_resize_right_bottom", sid.index())),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
            if response.dragged() {
                let delta = response.drag_delta();
                schematic.size.0 += delta.x.into();
                schematic.size.1 += delta.y.into();
            }
            let response = ui
                .interact(
                    egui::Rect::from_center_size(outer_rect.right_top(), egui::vec2(5.0, 5.0)),
                    Id::new(("schematic_resize_right_top", sid.index())),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNeSw);
            if response.dragged() {
                let delta = response.drag_delta();
                schematic.origin.1 += delta.y.into();
                schematic.size.0 += delta.x.into();
            }
            let response = ui
                .interact(
                    egui::Rect::from_center_size(outer_rect.left_bottom(), egui::vec2(5.0, 5.0)),
                    Id::new(("schematic_resize_left_bottom", sid.index())),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNeSw);
            if response.dragged() {
                let delta = response.drag_delta();
                schematic.origin.0 += delta.x.into();
                schematic.size.0 -= delta.x.into();
                schematic.size.1 += delta.y.into();
            }
            let response = ui
                .interact(
                    egui::Rect::from_center_size(outer_rect.left_top(), egui::vec2(5.0, 5.0)),
                    Id::new(("schematic_resize_left_top", sid.index())),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
            if response.dragged() {
                let delta = response.drag_delta();
                schematic.origin.0 += delta.x.into();
                schematic.origin.1 += delta.y.into();
                schematic.size.0 -= delta.x.into();
                schematic.size.1 -= delta.y.into();
            }
            let response = ui
                .interact(
                    outer_rect.shrink(20.0),
                    Id::new(("schematic_drag", sid.index())),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::Grab);
            if response.dragged() {
                let delta = response.drag_delta();
                schematic.origin.0 += delta.x.into();
                schematic.origin.1 += delta.y.into();
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
            }
            // Draw a pin for each port, and add a label
            let mut port_positions = std::mem::take(&mut schematic.port_positions);
            for (&port_id, port_position) in port_positions.iter_mut() {
                let port = schematic.find_port_by_id(port_id).unwrap();
                let is_input = schematic.is_port_input(port_id);
                let pin_name = if is_input {
                    format!("i{:?}", port.path)
                } else {
                    format!("o{:?}", port.path)
                };
                let mut flip = false;
                let pin_position = schematic.port_position(*port_position);
                let pin_position = pos2(pin_position.0.into(), pin_position.1.into());
                match port_position {
                    PortPosition::West(offset) => {
                        match Self::draw_pin(
                            ui,
                            port_id,
                            pin_position,
                            &pin_name,
                            egui::Align2::LEFT_CENTER,
                        ) {
                            Some(PortAction::Click) => {
                                self.pin_drag = Some(port_id);
                            }
                            Some(PortAction::Drag(pos, delta)) => {
                                if pos.x > outer_rect.center().x {
                                    flip = true;
                                }
                                *offset += delta.y.into();
                                *offset = (*offset).clamp(
                                    -(schematic.size.1 / 2.0 - 10.0),
                                    schematic.size.1 / 2.0 - 10.0,
                                );
                            }
                            _ => {}
                        }
                        if self.pin_drag == Some(port_id) {
                            ui.painter().circle_stroke(
                                pin_position,
                                4.0,
                                (1.0, Color32::LIGHT_BLUE),
                            );
                        }
                    }
                    PortPosition::East(offset) => {
                        match Self::draw_pin(
                            ui,
                            port_id,
                            pin_position,
                            &pin_name,
                            egui::Align2::RIGHT_CENTER,
                        ) {
                            Some(PortAction::Click) => {
                                self.pin_drag = Some(port_id);
                            }
                            Some(PortAction::Drag(pos, delta)) => {
                                if pos.x < outer_rect.center().x {
                                    flip = true;
                                }
                                *offset += delta.y.into();
                                *offset = (*offset).clamp(
                                    -(schematic.size.1 / 2.0 - 10.0),
                                    schematic.size.1 / 2.0 - 10.0,
                                );
                            }
                            _ => {}
                        }
                        if self.pin_drag == Some(port_id) {
                            ui.painter().circle_stroke(
                                pin_position,
                                4.0,
                                (1.0, Color32::LIGHT_BLUE),
                            );
                        }
                    }
                }
                if flip {
                    *port_position = match port_position {
                        PortPosition::East(offset) => PortPosition::West(*offset),
                        PortPosition::West(offset) => PortPosition::East(*offset),
                    };
                }
            }
            schematic.port_positions = port_positions;
        }
    }
    fn draw_schematic(&mut self, ui: &mut Ui) {
        let scene = Scene::new().zoom_range(0.1..=2.0);
        let mut scene_rect = self.scene_rect;
        scene.show(ui, &mut scene_rect, |ui| {
            ui.painter()
                .vline(0.0, -1000.0..=1000.0, (1.0, Color32::DARK_GRAY));
            ui.painter()
                .hline(-1000.0..=1000.0, 0.0, (1.0, Color32::DARK_GRAY));
            let sid = self.top_schematic_id();
            self.draw_plan_view(sid, ui);
        });
        self.scene_rect = scene_rect;
    }
    fn draw_schematic_detail(&mut self, ui: &mut Ui) {
        let scene = Scene::new().zoom_range(0.1..=2.0);
        let mut scene_rect = self.detail_scene_rect;
        scene.show(ui, &mut scene_rect, |ui| {
            ui.painter()
                .vline(0.0, -1000.0..=1000.0, (1.0, Color32::DARK_GRAY));
            ui.painter()
                .hline(-1000.0..=1000.0, 0.0, (1.0, Color32::DARK_GRAY));
            let sid = self.top_schematic_id();
            self.draw_schematic_plan_view(sid, ui);
        });
        self.detail_scene_rect = scene_rect;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let set = SchematicSet {
            schematics: self.schematic.clone(),
            loop_ports: self.highlighted_ports.iter().cloned().collect(),
        };
        let content = serde_json::to_string_pretty(&set).unwrap();
        std::fs::write(&self.filename, content).expect("Failed to write file");
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            "scene_rect",
            serde_json::to_string_pretty(&self.scene_rect).unwrap(),
        );
        storage.set_string(
            "detail_scene_rect",
            serde_json::to_string_pretty(&self.detail_scene_rect).unwrap(),
        );
    }
    fn persist_egui_memory(&self) -> bool {
        true
    }
}

fn main() -> Result {
    let args = Args::parse();
    let mut app = App::new(args.schematic);
    eframe::run_native(
        "Schematic View",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            if let Some(storage) = cc.storage {
                if let Some(scene_rect_str) = storage.get_string("scene_rect")
                    && let Ok(scene_rect) = serde_json::from_str::<egui::Rect>(&scene_rect_str)
                {
                    app.scene_rect = scene_rect;
                }
                if let Some(detail_scene_rect_str) = storage.get_string("detail_scene_rect")
                    && let Ok(detail_scene_rect) =
                        serde_json::from_str::<egui::Rect>(&detail_scene_rect_str)
                {
                    app.detail_scene_rect = detail_scene_rect;
                }
            }
            Ok(Box::new(app))
        }),
    )
}
