use super::utils;
use super::{EmuState, Immediate};

use std::collections::BTreeMap;

use imgui::{ImGuiCol, ImGuiCond, ImString, Ui};

pub struct DisasmWindow {
    disasm: BTreeMap<u16, String>,
    goto_pc: bool,
    goto_addr: ImString,
}

impl DisasmWindow {
    pub fn new(state: &EmuState) -> DisasmWindow {
        let mut dw = DisasmWindow {
            disasm: BTreeMap::new(),
            goto_pc: false,
            goto_addr: ImString::with_capacity(4),
        };

        dw.goto_addr.push_str("0000");
        dw.realign_disasm(state, 0);
        dw
    }

    // If there is alread an instruction decoded at address `from`, do nothing.
    // Otherwise, fetch the instruction at from, invalidate all the overlapping
    // instructions and update the disassembly. Do this until it's aligned again.
    fn realign_disasm(&mut self, state: &EmuState, mut from: u16) {
        let cpu = state.gb.cpu();
        let bus = state.gb.bus();

        while from < bus.rom_size() {
            let instr = cpu.disasm(bus, from);
            let next = from + u16::from(instr.size);

            if self.disasm.get(&from).is_some() {
                break;
            }
            for addr in from..next {
                self.disasm.remove(&addr);
            }

            self.disasm.insert(
                from,
                format!(
                    "{:04X}:  {:02X} {:5}    {}",
                    from,
                    instr.opcode,
                    match instr.imm {
                        Some(Immediate::Imm8(d8)) => format!("{:02X}", d8),
                        Some(Immediate::Imm16(d16)) => {
                            format!("{:02X} {:02X}", d16 & 0xFF, d16 >> 8)
                        }
                        None => String::new(),
                    },
                    instr.mnemonic
                ),
            );
            from = next;
        }
    }

    pub fn draw(&mut self, ui: &Ui, state: &mut EmuState) {
        let pc = state.gb.cpu().pc;

        // 99% of the time this does nothing, so it's cool
        // to have it called every rendering loop.
        self.realign_disasm(state, pc);

        ui.window(im_str!("ROM00 disassembly"))
            .size((300.0, 650.0), ImGuiCond::FirstUseEver)
            .position((10.0, 30.0), ImGuiCond::FirstUseEver)
            .build(|| {
                /*
                 * Disassembly listing
                 */
                ui.child_frame(im_str!("listing"), (285.0, 585.0))
                    .always_show_vertical_scroll_bar(true)
                    .show_borders(false)
                    .build(|| {
                        if self.goto_pc {
                            for (i, addr) in self.disasm.keys().enumerate() {
                                if *addr == pc {
                                    unsafe {
                                        imgui_sys::igSetScrollY(
                                            ui.get_text_line_height_with_spacing() * i as f32,
                                        );
                                    }
                                    break;
                                }
                            }
                        }

                        utils::list_clipper(ui, self.disasm.len(), |range| {
                            let instrs = self
                                .disasm
                                .iter()
                                .skip(range.start)
                                .take(range.end - range.start);

                            for (addr, instr) in instrs {
                                if *addr == pc {
                                    ui.with_color_var(ImGuiCol::Text, (0.0, 1.0, 0.0, 1.0), || {
                                        ui.text(instr)
                                    });
                                } else {
                                    ui.text(instr);
                                }
                            }
                        });
                    });

                ui.separator();

                /*
                 * GOTO logic
                 */
                ui.push_item_width(35.0);
                ui.input_text(im_str!("Goto"), &mut self.goto_addr)
                    .chars_hexadecimal(true)
                    .chars_noblank(true)
                    .chars_uppercase(true)
                    .auto_select_all(true)
                    .build();
                ui.pop_item_width();

                ui.same_line(230.0);

                self.goto_pc = ui.button(im_str!("Goto PC"), (0.0, 0.0));
            });
    }
}
