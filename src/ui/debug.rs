use super::utils;
use super::EmuState;

use imgui::{ImGuiCond, ImStr, ImString, ImVec2, Ui};

pub struct DebuggerWindow;

impl DebuggerWindow {
    pub fn new() -> DebuggerWindow {
        DebuggerWindow {}
    }

    pub fn draw(&self, ui: &Ui, state: &mut EmuState) {
        ui.window(im_str!("Debugger"))
            .size((450.0, 140.0), ImGuiCond::FirstUseEver)
            .position((320.0, 30.0), ImGuiCond::FirstUseEver)
            .build(|| {
                let cpu = state.gb.cpu();

                DebuggerWindow::draw_reg(ui, "AF", cpu.af);
                ui.same_line(0.0);
                DebuggerWindow::draw_reg(ui, "BC", cpu.bc);
                ui.same_line(0.0);
                DebuggerWindow::draw_reg(ui, "DE", cpu.de);
                ui.same_line(0.0);
                DebuggerWindow::draw_reg(ui, "HL", cpu.hl);

                DebuggerWindow::draw_reg(ui, "SP", cpu.sp);
                ui.same_line(0.0);
                DebuggerWindow::draw_reg(ui, "PC", cpu.pc);

                ui.separator();

                ui.checkbox(im_str!("Break"), &mut state.stepping);
                ui.same_line(0.0);
                state.step_into = ui.button(im_str!("Step"), (0.0, 0.0));
                if state.step_into {
                    state.stepping = true;
                }
                ui.checkbox(
                    im_str!("Break on invalid opcode"),
                    &mut state.break_on_invalid,
                );
            });
    }

    fn draw_reg(ui: &Ui, s: &str, val: u16) {
        let mut val = ImString::from(format!("{:04X}", val));

        ui.push_item_width(35.0);

        ui.input_text(ImStr::new(&ImString::from(String::from(s))), &mut val)
            .read_only(true)
            .build();

        ui.pop_item_width();
    }

    // fn print_flags(ui: &Ui, state: &EmuState) {
    //     let cpu = state.gb.cpu();
    //     let bg_col = utils::text_bg_color(ui);

    //     for (i, (n, f)) in [
    //         ("Z", cpu.zf()),
    //         ("N", cpu.sf()),
    //         ("H", cpu.hc()),
    //         ("C", cpu.cy()),
    //     ]
    //     .iter()
    //     .enumerate()
    //     {
    //         let x = 100.0 + (i as f32 * 20.0);

    //         utils::text_with_bg(
    //             ui,
    //             (x, 105.0),
    //             ImString::new(*n),
    //             if *f { Some(bg_col) } else { None },
    //         );
    //     }
    // }
}
