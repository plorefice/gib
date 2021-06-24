use super::utils;
use super::EmuState;
use super::WindowView;

use imgui::{im_str, Condition, Ui};

pub struct PeripheralView;

impl PeripheralView {
    pub fn new() -> PeripheralView {
        PeripheralView
    }
}

impl WindowView for PeripheralView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool {
        let mut open = true;

        ui.window(im_str!("Peripherals"))
            .size([310.0, 650.0], Condition::FirstUseEver)
            .position([955.0, 30.0], Condition::FirstUseEver)
            .opened(&mut open)
            .build(|| {
                if ui.collapsing_header(im_str!("Video Display")).build() {
                    ui.text("NOT IMPLEMENTED YET!");
                }

                if ui
                    .collapsing_header(im_str!("Sound Controller"))
                    .default_open(true)
                    .build()
                {
                    self.draw_sound_controller(ui, state);
                }

                if ui.collapsing_header(im_str!("Joypad Input")).build() {
                    ui.text("NOT IMPLEMENTED YET!");
                }

                if ui.collapsing_header(im_str!("Link Cable")).build() {
                    ui.text("NOT IMPLEMENTED YET!");
                }

                if ui
                    .collapsing_header(im_str!("Timer and Divider"))
                    .default_open(true)
                    .build()
                {
                    self.draw_timer(ui, state);
                }

                if ui
                    .collapsing_header(im_str!("Interrupts"))
                    .default_open(true)
                    .build()
                {
                    self.draw_interrupts(ui, state);
                }
            });

        open
    }
}

impl PeripheralView {
    fn draw_sound_controller(&self, ui: &Ui, state: &EmuState) {
        let apu = &state.bus().apu;

        // Sweep channel rendering
        {
            ui.text("Sweep Channel");

            ui.same_line(150.0);
            ui.text_colored(
                if apu.ch1.enabled() {
                    utils::GREEN
                } else {
                    utils::DARK_GREEN
                },
                im_str!("ENABLED"),
            );

            ui.same_line(220.0);
            ui.text_colored(
                if apu.ch1.dac_on() {
                    utils::GREEN
                } else {
                    utils::DARK_GREEN
                },
                im_str!("DAC"),
            );

            ui.separator();
        }

        ui.spacing();

        // Tone channel rendering
        {
            ui.text("Tone Channel");

            ui.same_line(150.0);
            ui.text_colored(
                if apu.ch1.enabled() {
                    utils::GREEN
                } else {
                    utils::DARK_GREEN
                },
                im_str!("ENABLED"),
            );

            ui.same_line(220.0);
            ui.text_colored(
                if apu.ch1.dac_on() {
                    utils::GREEN
                } else {
                    utils::DARK_GREEN
                },
                im_str!("DAC"),
            );

            ui.separator();
        }

        ui.spacing();

        // Wave channel rendering
        {
            ui.text("Wave Channel");
            ui.separator();
        }

        ui.spacing();

        // Noise channel rendering
        {
            ui.text("Noise Channel");
            ui.separator();
        }
    }

    fn draw_timer(&self, ui: &Ui, state: &EmuState) {
        let timer = &state.bus().tim;

        utils::input_addr(ui, "DIV", &mut Some(timer.sys_counter.0), false);
        ui.same_line(0.0);
        utils::input_addr(ui, "TIMA", &mut Some(u16::from(timer.tima.0)), false);
        ui.same_line(0.0);
        utils::input_addr(ui, "TMA", &mut Some(u16::from(timer.tma.0)), false);

        ui.separator();

        let rate = match timer.tac.0 & 0x3 {
            0b00 => "  4096 Hz",
            0b01 => "262144 Hz",
            0b10 => " 65536 Hz",
            0b11 => " 16384 Hz",
            _ => unreachable!(),
        };

        ui.text(format!("Clock: {}", rate));

        ui.same_line_with_spacing(0.0, 40.0);

        ui.text_colored(
            if (timer.tac.0 & 0x4) != 0 {
                utils::GREEN
            } else {
                utils::DARK_GREEN
            },
            "RUNNING",
        );
    }

    fn draw_interrupts(&self, ui: &Ui, state: &EmuState) {
        let itr = &state.bus().itr;
        let irqs = [
            (0, "BLANK"),
            (1, "STAT"),
            (2, "TIM"),
            (3, "SER"),
            (4, "JOY"),
        ];

        ui.text("IE:");

        for (b, s) in irqs.iter() {
            ui.same_line_with_spacing(0.0, 15.0);
            ui.text_colored(
                if itr.ien.bit(*b) {
                    utils::GREEN
                } else {
                    utils::DARK_GREEN
                },
                s,
            );
        }

        ui.text("IF:");

        for (b, s) in irqs.iter() {
            ui.same_line_with_spacing(0.0, 15.0);
            ui.text_colored(
                if itr.ifg.bit(*b) {
                    utils::GREEN
                } else {
                    utils::DARK_GREEN
                },
                s,
            );
        }
    }
}
