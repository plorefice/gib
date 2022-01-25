use imgui::{ChildWindow, CollapsingHeader, Condition, Ui, Window};

use crate::ui::{state::EmuState, utils};

use super::WindowView;

pub struct DebuggerView;

impl DebuggerView {
    pub fn new() -> DebuggerView {
        DebuggerView
    }
}

impl DebuggerView {
    fn draw_cpu_state(&mut self, ui: &Ui, state: &EmuState) {
        let cpu = state.cpu();

        ui.text(format!(
            "Clock cycle: {:12}",
            state.gameboy().clock_cycles()
        ));

        if *cpu.halted.value() {
            ui.same_line_with_spacing(0.0, 20.0);
            ui.text_colored(utils::RED, "HALT");
        }

        if *cpu.intr_enabled.value() {
            ui.same_line_with_spacing(0.0, 20.0);
            ui.text_colored(utils::GREEN, "IME");
        }

        ui.separator();

        utils::input_addr(ui, "AF", &mut Some(cpu.af), false);
        ui.same_line();
        utils::input_addr(ui, "BC", &mut Some(cpu.bc), false);
        ui.same_line();
        utils::input_addr(ui, "DE", &mut Some(cpu.de), false);
        ui.same_line();
        utils::input_addr(ui, "HL", &mut Some(cpu.hl), false);
        ui.same_line();
        utils::input_addr(ui, "SP", &mut Some(cpu.sp), false);
        ui.same_line();
        utils::input_addr(ui, "PC", &mut Some(cpu.pc), false);

        ui.text(format!(
            "Flags: {} {} {} {}",
            if cpu.zf() { 'Z' } else { '-' },
            if cpu.sf() { 'N' } else { '-' },
            if cpu.hc() { 'H' } else { '-' },
            if cpu.cy() { 'C' } else { '-' },
        ));

        ui.same_line_with_pos(150.0);

        if let Some(ref evt) = state.last_event() {
            ui.text_colored(utils::RED, evt.to_string());
        } else {
            ui.text("");
        }
    }

    fn draw_call_stack(&mut self, ui: &Ui, state: &EmuState) {
        ChildWindow::new("callstack_frame")
            .size([125.0, 0.0])
            .build(ui, || {
                if CollapsingHeader::new("Call Stack")
                    .default_open(true)
                    .build(ui)
                {
                    for (i, addr) in state.cpu().call_stack.iter().rev().enumerate() {
                        let c = if i == 0 {
                            utils::WHITE
                        } else {
                            utils::DARK_GREY
                        };

                        ui.text_colored(
                            c,
                            format!(" {} 0x{:04X}", if i == 0 { '>' } else { ' ' }, addr),
                        );
                    }
                }
            });
    }
}

impl WindowView for DebuggerView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool {
        let mut open = true;

        Window::new("Debugger")
            .size([390.0, 240.0], Condition::FirstUseEver)
            .position([320.0, 30.0], Condition::FirstUseEver)
            .opened(&mut open)
            .build(ui, || {
                self.draw_cpu_state(ui, state);

                ui.separator();

                if ui.button("Run") {
                    state.set_running();
                }
                ui.same_line();

                if ui.button("Pause") {
                    state.pause();
                }
                ui.same_line();

                if ui.button("Step") {
                    state.set_single_step();
                }

                ui.separator();

                self.draw_call_stack(ui, state);
            });

        open
    }
}
