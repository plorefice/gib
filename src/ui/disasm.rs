use super::GameBoy;

use imgui::{ImGuiCond, Ui};

pub struct DisasmWindow {
    disasm: Vec<String>,
}

impl DisasmWindow {
    pub fn new(emu: &GameBoy) -> DisasmWindow {
        let mut disasm = Vec::with_capacity(0x4000);
        let mut addr = 0u16;

        while addr < 0x4000 {
            let (s, sz) = emu.cpu().disasm(emu.bus(), addr);
            addr += u16::from(sz);
            disasm.push(s);
        }

        DisasmWindow { disasm }
    }

    pub fn draw(&self, ui: &Ui) -> bool {
        ui.window(im_str!("ROM00 disassembly"))
            .size((300.0, 700.0), ImGuiCond::FirstUseEver)
            .build(|| {
                for s in self.disasm.iter() {
                    ui.text(s);
                }
            });

        true
    }
}
