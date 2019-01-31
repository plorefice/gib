use super::utils;
use super::EmuState;
use super::WindowView;

use imgui::{ImGuiCol, ImGuiCond, ImStr, ImString, Ui};

pub struct DebuggerView;

impl DebuggerView {
    pub fn new() -> DebuggerView {
        DebuggerView
    }

    fn draw_reg(ui: &Ui, s: &str, val: u16) {
        let mut val = ImString::from(format!("{:04X}", val));

        ui.push_item_width(37.0);

        ui.input_text(ImStr::new(&ImString::from(String::from(s))), &mut val)
            .read_only(true)
            .build();

        ui.pop_item_width();
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
                let cpu = state.gb.cpu();

                ui.text(format!("Clock cycle: {:12}", cpu.clk));

                ui.separator();

                DebuggerView::draw_reg(ui, "AF", cpu.af);
                ui.same_line(0.0);
                DebuggerView::draw_reg(ui, "BC", cpu.bc);
                ui.same_line(0.0);
                DebuggerView::draw_reg(ui, "DE", cpu.de);
                ui.same_line(0.0);
                DebuggerView::draw_reg(ui, "HL", cpu.hl);
                ui.same_line(0.0);
                DebuggerView::draw_reg(ui, "SP", cpu.sp);
                ui.same_line(0.0);
                DebuggerView::draw_reg(ui, "PC", cpu.pc);

                ui.text(format!(
                    "Flags: {} {} {} {}",
                    if cpu.zf() { 'Z' } else { '-' },
                    if cpu.sf() { 'N' } else { '-' },
                    if cpu.hc() { 'H' } else { '-' },
                    if cpu.cy() { 'C' } else { '-' },
                ));

                ui.same_line(150.0);

                if let Some(ref evt) = state.trace_event {
                    ui.with_color_var(ImGuiCol::Text, utils::RED, || {
                        ui.text(evt.to_string());
                    });
                } else {
                    ui.text("");
                }

                ui.separator();

                ui.checkbox(im_str!("Break"), &mut state.stepping);
                ui.same_line(0.0);
                state.step_into = ui.button(im_str!("Step"), (0.0, 0.0));
                if state.step_into {
                    state.stepping = true;
                }
                ui.same_line_spacing(0.0, 110.0);
                ui.checkbox(im_str!("Break on exception"), &mut state.break_on_exception);
            });

        open
    }
}
