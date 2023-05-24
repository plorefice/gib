use egui::Color32;

use crate::ui::{state::Emulator, utils};

#[derive(Default)]
pub struct Debugger {
    registers: [String; 6],
}

impl super::Window for Debugger {
    fn name(&self) -> &'static str {
        "Debugger"
    }

    fn show(&mut self, ctx: &egui::Context, state: &mut Emulator, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_pos([330.0, 30.0])
            .show(ctx, |ui| {
                use super::View as _;
                self.ui(ui, state);
            });
    }
}

impl super::View for Debugger {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut Emulator) {
        egui::Grid::new("debugger-grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    self.left_column_ui(ui, state);
                });
                ui.vertical(|ui| {
                    self.call_stack_ui(ui, state);
                });
            });
    }
}

impl Debugger {
    fn left_column_ui(&mut self, ui: &mut egui::Ui, state: &mut Emulator) {
        self.cpu_state_ui(ui, state);

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Run").clicked() {
                state.set_running();
            }
            if ui.button("Pause").clicked() {
                state.pause();
            }
            if ui.button("Step").clicked() {
                state.set_single_step();
            }
        });

        ui.separator();

        if let Some(ref evt) = state.last_event() {
            ui.colored_label(Color32::RED, evt.to_string());
        } else {
            ui.label(egui::RichText::new("No event to report").weak());
        }
    }

    fn cpu_state_ui(&mut self, ui: &mut egui::Ui, state: &Emulator) {
        let cpu = state.cpu();

        ui.horizontal(|ui| {
            ui.label(format!(
                "Clock cycle: {:12}",
                state.gameboy().clock_cycles()
            ));

            ui.add_space(5.);
            ui.colored_label(
                if *cpu.halted.value() {
                    Color32::RED
                } else {
                    Color32::from_rgb(50, 0, 0)
                },
                "HALT",
            );

            ui.add_space(5.);
            ui.colored_label(
                if *cpu.intr_enabled.value() {
                    Color32::GREEN
                } else {
                    Color32::from_rgb(0, 50, 0)
                },
                "IME",
            );
        });

        ui.separator();

        // Update register state
        for (reg, buf) in [cpu.af, cpu.bc, cpu.de, cpu.hl, cpu.sp, cpu.pc]
            .into_iter()
            .zip(self.registers.iter_mut())
        {
            *buf = format!("{reg:04X}");
        }

        egui::Grid::new("debugger-registers")
            .num_columns(3)
            .spacing([5., 2.])
            .min_col_width(70.)
            .show(ui, |ui| {
                utils::address_edit_ui(ui, "AF", &mut self.registers[0], false);
                utils::address_edit_ui(ui, "BC", &mut self.registers[1], false);
                utils::address_edit_ui(ui, "DE", &mut self.registers[2], false);
                ui.end_row();
                utils::address_edit_ui(ui, "HL", &mut self.registers[3], false);
                utils::address_edit_ui(ui, "SP", &mut self.registers[4], false);
                utils::address_edit_ui(ui, "PC", &mut self.registers[5], false);
            });

        ui.label(format!(
            "Flags: {} {} {} {}",
            if cpu.zf() { 'Z' } else { '-' },
            if cpu.sf() { 'N' } else { '-' },
            if cpu.hc() { 'H' } else { '-' },
            if cpu.cy() { 'C' } else { '-' },
        ));
    }

    fn call_stack_ui(&mut self, ui: &mut egui::Ui, state: &Emulator) {
        egui::CollapsingHeader::new("Call stack")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (i, addr) in state.cpu().call_stack.iter().rev().enumerate() {
                            let c = if i == 0 {
                                Color32::WHITE
                            } else {
                                Color32::DARK_GRAY
                            };

                            ui.colored_label(c, format!("0x{addr:04X}"));
                        }
                    });
            });
    }
}
