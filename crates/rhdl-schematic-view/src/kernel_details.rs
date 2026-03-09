use std::collections::HashSet;

use egui::Ui;
use rhdl_core::circuit::schematic::{PortId, Schematic, SchematicKind};

use crate::{schematic_rendering::has_highlighted_ports, slot};

#[derive(Default)]
pub struct KernelDetails {
    pub name: String,
    pub filename: String,
    pub table: Vec<KernelTableEntry>,
}

impl egui::Widget for &KernelDetails {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let font = egui::FontId::monospace(10.0);
        egui::Frame::new()
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(format!("Name: {}", self.name)).font(font.clone()));
                ui.label(
                    egui::RichText::new(format!("Filename: {}", self.filename)).font(font.clone()),
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

pub struct KernelTableEntry {
    pub inputs: String,
    pub opcode: String,
    pub outputs: String,
    pub source_code: String,
}

pub fn render_kernel(schematic: &Schematic, highlighted_ports: &HashSet<PortId>) -> KernelDetails {
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
