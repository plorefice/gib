use egui::Color32;

use crate::ui::{state::Emulator, utils};

#[derive(Default)]
pub struct Peripherals;

impl super::Window for Peripherals {
    fn name(&self) -> &'static str {
        "Peripherals"
    }

    fn show(&mut self, ctx: &egui::Context, state: &mut Emulator, open: &mut bool) {
        egui::Window::new(self.name())
            .default_pos([915.0, 30.0])
            .default_size([310.0, 650.0])
            .open(open)
            .show(ctx, |ui| {
                use super::View;
                self.ui(ui, state);
            });
    }
}

impl super::View for Peripherals {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut Emulator) {
        egui::CollapsingHeader::new("Video Display").show(ui, |ui| {
            ui.label("NOT IMPLEMENTED YET!");
        });

        egui::CollapsingHeader::new("Sound Controller")
            .default_open(true)
            .show(ui, |ui| {
                self.sound_controller_ui(ui, state);
            });

        egui::CollapsingHeader::new("Joypad Input").show(ui, |ui| {
            ui.label("NOT IMPLEMENTED YET!");
        });

        egui::CollapsingHeader::new("Link Cable").show(ui, |ui| {
            ui.label("NOT IMPLEMENTED YET!");
        });

        egui::CollapsingHeader::new("Timer and Divider")
            .default_open(true)
            .show(ui, |ui| {
                self.timers_ui(ui, state);
            });

        egui::CollapsingHeader::new("Interrupts")
            .default_open(true)
            .show(ui, |ui| {
                self.interrupts_ui(ui, state);
            });
    }
}

impl Peripherals {
    fn sound_controller_ui(&self, ui: &mut egui::Ui, state: &Emulator) {
        let apu = &state.bus().apu;

        egui::Grid::new("sweep-channel")
            .num_columns(2)
            .min_col_width(150.)
            .show(ui, |ui| {
                ui.label("Sweep Channel");

                ui.horizontal(|ui| {
                    ui.colored_label(
                        if apu.ch1.enabled() {
                            Color32::GREEN
                        } else {
                            Color32::DARK_GREEN
                        },
                        "ENABLED",
                    );

                    ui.add_space(30.);

                    ui.colored_label(
                        if apu.ch1.dac_on() {
                            Color32::GREEN
                        } else {
                            Color32::DARK_GREEN
                        },
                        "DAC",
                    );
                });
            });

        ui.separator();

        egui::Grid::new("tone-channel")
            .num_columns(2)
            .min_col_width(150.)
            .show(ui, |ui| {
                ui.label("Tone Channel");

                ui.horizontal(|ui| {
                    ui.colored_label(
                        if apu.ch1.enabled() {
                            Color32::GREEN
                        } else {
                            Color32::DARK_GREEN
                        },
                        "ENABLED",
                    );

                    ui.add_space(30.);

                    ui.colored_label(
                        if apu.ch1.dac_on() {
                            Color32::GREEN
                        } else {
                            Color32::DARK_GREEN
                        },
                        "DAC",
                    );
                });
            });

        ui.separator();

        ui.label("Wave Channel");

        ui.separator();

        ui.label("Noise Channel");
    }

    fn timers_ui(&self, ui: &mut egui::Ui, state: &Emulator) {
        let timer = &state.bus().tim;

        ui.horizontal(|ui| {
            utils::address_edit_ui(ui, "DIV", &mut utils::hexify(timer.sys_counter.0), false);
            utils::address_edit_ui(ui, "TIMA", &mut utils::hexify(timer.tima.0), false);
            utils::address_edit_ui(ui, "TMA", &mut utils::hexify(timer.tma.0), false);
        });

        ui.separator();

        let rate = match timer.tac.0 & 0x3 {
            0b00 => "  4096 Hz",
            0b01 => "262144 Hz",
            0b10 => " 65536 Hz",
            0b11 => " 16384 Hz",
            _ => unreachable!(),
        };

        ui.horizontal(|ui| {
            ui.label(format!("Clock: {rate}"));
            ui.add_space(40.0);
            ui.colored_label(
                if (timer.tac.0 & 0x4) != 0 {
                    Color32::GREEN
                } else {
                    Color32::DARK_GREEN
                },
                "RUNNING",
            );
        });
    }

    fn interrupts_ui(&self, ui: &mut egui::Ui, state: &Emulator) {
        let itr = &state.bus().itr;
        let irqs = [
            (0, "BLANK"),
            (1, "STAT"),
            (2, "TIM"),
            (3, "SER"),
            (4, "JOY"),
        ];

        ui.horizontal(|ui| {
            ui.label("IE:");

            for &(b, s) in irqs.iter() {
                ui.add_space(15.0);
                ui.colored_label(
                    if itr.ien.bit(b) {
                        Color32::GREEN
                    } else {
                        Color32::DARK_GREEN
                    },
                    s,
                );
            }
        });

        ui.horizontal(|ui| {
            ui.label("IF:");

            for &(b, s) in irqs.iter() {
                ui.add_space(15.0);
                ui.colored_label(
                    if itr.ifg.bit(b) {
                        Color32::GREEN
                    } else {
                        Color32::DARK_GREEN
                    },
                    s,
                );
            }
        });
    }
}
