use super::EmuState;

use std::collections::BTreeMap;

use imgui::{ImGuiCol, ImGuiCond, Ui};

pub struct DisasmWindow {
    disasm: BTreeMap<u16, String>,
}

impl DisasmWindow {
    pub fn new(state: &EmuState) -> DisasmWindow {
        let mut dw = DisasmWindow {
            disasm: BTreeMap::new(),
        };

        dw.realign_disasm(state, 0);
        dw
    }

    // If there is alread an instruction decoded at address `from`,
    // do nothing. Otherwise, fetch the instruction at from, invalidate
    // all the overlapping decoded instructions and update the view.
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
                    "{:04X}\t{:02X} {:4}\t{}",
                    from,
                    instr.opcode,
                    if let Some(imm) = instr.imm {
                        format!("{:04X}", imm)
                    } else {
                        "    ".to_string()
                    },
                    instr.mnemonic
                ),
            );
            from = next;
        }
    }

    pub fn draw(&mut self, ui: &Ui, state: &mut EmuState) {
        let pc = state.gb.cpu().pc;

        self.realign_disasm(state, pc);

        ui.window(im_str!("ROM00 disassembly"))
            .size((300.0, 700.0), ImGuiCond::FirstUseEver)
            .position((10.0, 30.0), ImGuiCond::FirstUseEver)
            .build(|| {
                for (addr, instr) in self.disasm.iter() {
                    if *addr == pc {
                        ui.with_color_var(ImGuiCol::Text, (0.0, 1.0, 0.0, 1.0), || ui.text(instr));
                    } else {
                        ui.text(instr);
                    }
                }
            });
    }
}
