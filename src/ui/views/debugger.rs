use super::utils;
use super::EmuState;
use super::WindowView;

use imgui::{ImGuiCol, ImGuiCond, Ui};

pub struct DebuggerView;

impl DebuggerView {
    pub fn new() -> DebuggerView {
        DebuggerView
    }
}

impl WindowView for DebuggerView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool {
        let mut open = true;

        ui.window(im_str!("Debugger"))
            .size((390.0, 120.0), ImGuiCond::FirstUseEver)
            .position((320.0, 30.0), ImGuiCond::FirstUseEver)
            .opened(&mut open)
            .build(|| {
                let cpu = state.cpu();

                ui.text(format!("Clock cycle: {:12}", cpu.clk));

                if cpu.halted {
                    ui.same_line_spacing(0.0, 10.0);
                    ui.with_color_var(ImGuiCol::Text, utils::RED, || {
                        ui.text(im_str!("HALT"));
                    });
                }

                ui.separator();

                utils::input_addr(ui, "AF", &mut Some(cpu.af), false);
                ui.same_line(0.0);
                utils::input_addr(ui, "BC", &mut Some(cpu.bc), false);
                ui.same_line(0.0);
                utils::input_addr(ui, "DE", &mut Some(cpu.de), false);
                ui.same_line(0.0);
                utils::input_addr(ui, "HL", &mut Some(cpu.hl), false);
                ui.same_line(0.0);
                utils::input_addr(ui, "SP", &mut Some(cpu.sp), false);
                ui.same_line(0.0);
                utils::input_addr(ui, "PC", &mut Some(cpu.pc), false);

                ui.text(format!(
                    "Flags: {} {} {} {}",
                    if cpu.zf() { 'Z' } else { '-' },
                    if cpu.sf() { 'N' } else { '-' },
                    if cpu.hc() { 'H' } else { '-' },
                    if cpu.cy() { 'C' } else { '-' },
                ));

                ui.same_line(150.0);

                if let Some(ref evt) = state.last_event() {
                    ui.with_color_var(ImGuiCol::Text, utils::RED, || {
                        ui.text(evt.to_string());
                    });
                } else {
                    ui.text("");
                }

                ui.separator();

                if ui.button(im_str!("Run"), (0.0, 0.0)) {
                    state.set_running();
                }
                ui.same_line(0.0);

                if ui.button(im_str!("Pause"), (0.0, 0.0)) {
                    state.pause();
                }
                ui.same_line(0.0);

                if ui.button(im_str!("Step"), (0.0, 0.0)) {
                    state.set_single_step();
                }
                ui.same_line(0.0);
            });

        open
    }
}
