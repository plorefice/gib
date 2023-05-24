use egui::Color32;
use gib_core::dbg::MemoryType;

use crate::ui::state::Emulator;

pub struct MemoryMap {
    map: Vec<(MemoryType, String)>,
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            map: MemoryType::default()
                .iter()
                .map(|mt| {
                    let r = mt.range();
                    (mt, format!("{:04X}-{:04X}    {mt}", r.start(), r.end()))
                })
                .collect(),
        }
    }
}

impl super::Window for MemoryMap {
    fn name(&self) -> &'static str {
        "Memory Map"
    }

    fn show(&mut self, ctx: &egui::Context, state: &mut Emulator, open: &mut bool) {
        egui::Window::new(self.name())
            .default_pos([915.0, 700.0])
            .default_size([225.0, 290.0])
            .open(open)
            .show(ctx, |ui| {
                use super::View;
                self.ui(ui, state);
            });
    }
}

impl super::View for MemoryMap {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut Emulator) {
        let pc = state.cpu().pc;

        for (mt, s) in &self.map {
            let color = if MemoryType::at(pc) == *mt {
                Color32::GREEN
            } else {
                Color32::WHITE
            };

            ui.colored_label(color, s);
        }
    }
}
