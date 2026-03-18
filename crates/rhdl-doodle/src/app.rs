use core::f32;
use std::path::PathBuf;

use egui::{DragPanButtons, Rect, Scene, TextEdit, pos2};
use egui_tiles::{Tree, UiResponse};

use crate::{drawing::Drawing, label::LabelSide};

enum Pane {
    Drawing,
    Properties,
}

pub struct App {
    filename: PathBuf,
    drawing: Drawing,
    pub scene_rect: Rect,
    viz_tree: Option<Tree<Pane>>,
}

impl App {
    pub fn new(filename: PathBuf) -> Self {
        let tree =
            egui_tiles::Tree::new_horizontal("viz_tree", vec![Pane::Drawing, Pane::Properties]);
        Self {
            filename,
            drawing: Drawing::default(),
            scene_rect: Rect::ZERO,
            viz_tree: Some(tree),
        }
    }
    fn properties(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::new().inner_margin(10.0);
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Properties");
                if let Some(rbox) = self.drawing.selected_rect_mut() {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut rbox.name);
                    });
                }
                let props = egui::Grid::new("prop_grid").num_columns(3).striped(true);
                props.show(ui, |ui| {
                    ui.label("Side");
                    ui.label("Offset");
                    ui.label("Text");
                    ui.end_row();
                    if let Some(rbox) = self.drawing.selected_rect_mut() {
                        for label in &mut rbox.labels {
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut label.side, LabelSide::West, "West");
                                ui.selectable_value(&mut label.side, LabelSide::East, "East");
                            });
                            ui.add(egui::DragValue::new(&mut label.offset).speed(1.0));
                            TextEdit::singleline(&mut label.text)
                                .desired_width(f32::INFINITY)
                                .show(ui);
                            ui.end_row();
                        }
                    }
                });
                if ui
                    .add_enabled(self.drawing.selected.is_some(), egui::Button::new("+Port"))
                    .clicked()
                {
                    self.drawing.add_new_label();
                }
            });
        });
    }
}

impl egui_tiles::Behavior<Pane> for App {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Drawing => {
                let scene = Scene::new()
                    .zoom_range(0.1..=4.0)
                    .drag_pan_buttons(DragPanButtons::SECONDARY);
                let response = scene.show(ui, &mut self.scene_rect, |ui| {
                    self.drawing.update_ro(ui);
                });
                self.drawing.update_state(response.response);
                /*                 let response = response.response;
                               if self.drawing.line_add_anchor.is_some()
                                   && let Some(pos2) = response.hover_pos()
                               {
                                   self.drawing.line_add_current = Some(pos2);
                               }
                               if response.drag_started_by(egui::PointerButton::Primary)
                                   && let Some(pos) = response.interact_pointer_pos()
                               {
                                   self.drag_id = Some(
                                       self.drawing
                                           .add_rect_box(pos, vec2(GRID_SIZE, GRID_SIZE))
                                           .id(),
                                   );
                                   self.drawing.set_selected(self.drag_id);
                                   self.drag_start_pos = Some(grid(pos));
                               }
                               if response.dragged_by(egui::PointerButton::Primary)
                                   && let Some(_drag_id) = self.drag_id
                                   && let Some(pos) = response.interact_pointer_pos()
                                   && let Some(start) = self.drag_start_pos
                                   && let Some(rbox) = self.drawing.selected_rect_mut()
                               {
                                   rbox.inner = Rect::from_two_pos(start, grid(pos));
                                   rbox.edit_kind = Some(SaveState {
                                       kind: ModificationKind::Create,
                                       orig: rbox.inner,
                                   });
                               }
                               if response.drag_stopped_by(egui::PointerButton::Primary)
                                   && let Some(rbox) = self.drawing.selected_rect_mut()
                               {
                                   let width = round_to_even_grid(rbox.inner.width());
                                   let height = round_to_even_grid(rbox.inner.height());
                                   rbox.inner.set_height(height);
                                   rbox.inner.set_width(width);
                                   rbox.complete_edit();
                                   if height <= 0.0 || width <= 0.0 {
                                       self.drawing.delete_selected();
                                   }
                                   self.drag_id = None;
                                   self.drag_start_pos = None;
                               }
                               if response.clicked_by(egui::PointerButton::Primary) {
                                   self.drawing.set_selected(None);
                               }
                */
            }
            Pane::Properties => {
                self.properties(ui);
            }
        }
        UiResponse::default()
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Drawing => "Drawing".into(),
            Pane::Properties => "Properties".into(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                if ui.button("+Rect").clicked() {
                    self.drawing
                        .add_rect_box(pos2(100.0, 100.0), pos2(250.0, 300.0));
                }
                let mut viz_tree = std::mem::take(&mut self.viz_tree).unwrap();
                viz_tree.ui(self, ui);
                self.viz_tree = Some(viz_tree);
            });
        });
    }
}
