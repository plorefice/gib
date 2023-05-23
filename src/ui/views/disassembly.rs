use std::{cmp::Ordering, collections::BTreeMap};

use egui::{Color32, RichText};
use gib_core::{cpu::Immediate, dbg};

use crate::ui::{state::EmuState, utils};

pub struct Disassembly {
    section: dbg::MemoryType,
    disasm: BTreeMap<u16, String>,
    follow_pc: bool,
    goto_addr: String,
}

impl Default for Disassembly {
    fn default() -> Self {
        Self {
            section: dbg::MemoryType::RomBank(0),
            disasm: BTreeMap::new(),
            follow_pc: false,
            goto_addr: String::new(),
        }
    }
}

impl super::Window for Disassembly {
    fn name(&self) -> &'static str {
        "Disassembly"
    }

    fn show(&mut self, ctx: &egui::Context, state: &mut EmuState, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_pos([10.0, 30.0])
            .default_size([300.0, 650.0])
            .show(ctx, |ui| {
                use super::View;
                self.ui(ui, state);
            });
    }
}

impl super::View for Disassembly {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut EmuState) {
        // Most of the times this call does nothing, so it's cool to have it called every frame
        self.realign_disasm(state, state.cpu().pc);

        let goto_addr = self.goto_bar_ui(ui, state);
        if let Some(addr) = goto_addr {
            self.realign_disasm(state, addr);
        }

        ui.separator();

        self.disassembly_ui(ui, state, goto_addr);
    }
}

impl Disassembly {
    /// If there is alread an instruction decoded at address `from`, do nothing.
    /// Otherwise, fetch the instruction at from, invalidate all the overlapping
    /// instructions and update the disassembly. Do this until it's aligned again.
    /// If `from` is outside the current memory space, swap it and reload disasm.
    fn realign_disasm(&mut self, state: &EmuState, mut from: u16) {
        let cpu = state.cpu();
        let bus = state.bus();

        if self.disasm.contains_key(&from) {
            return;
        }

        let mut mem_range = self.section.range();

        if !mem_range.contains(&from) {
            self.section = dbg::MemoryType::at(from);
            self.disasm.clear();

            mem_range = self.section.range();
            from = *mem_range.start();
        }

        while from < *mem_range.end() {
            let instr = match cpu.disasm(bus, from) {
                Ok(instr) => instr,
                Err(evt) => panic!("unexpected trace event during disassembly: {}", evt),
            };

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
                        Some(Immediate::Imm16(d16)) => format!("{:04X}", d16),
                        None => String::new(),
                    },
                    instr.mnemonic
                ),
            );
            from = next;
        }
    }

    fn goto_bar_ui(&mut self, ui: &mut egui::Ui, state: &EmuState) -> Option<u16> {
        ui.horizontal(|ui| {
            let goto_addr = utils::address_edit_ui(ui, "Address", &mut self.goto_addr, true);
            let goto_addr = ui.button("Goto").clicked() || goto_addr;

            let goto_pc = ui.button("Goto PC").clicked();

            ui.checkbox(&mut self.follow_pc, "Follow");

            // Build response
            if goto_addr {
                u16::from_str_radix(&self.goto_addr, 16).ok()
            } else if goto_pc || self.follow_pc {
                Some(state.cpu().pc)
            } else {
                None
            }
        })
        .inner
    }

    fn disassembly_ui(&mut self, ui: &mut egui::Ui, state: &mut EmuState, goto_addr: Option<u16>) {
        let pc = state.cpu().pc;

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .auto_shrink([false; 2])
            .always_show_scroll(true)
            .show(ui, |ui| {
                let cpu = state.cpu_mut();

                for (addr, instr) in &self.disasm {
                    let color = match addr.cmp(&pc) {
                        Ordering::Less => Color32::DARK_GRAY,
                        Ordering::Equal => Color32::GREEN,
                        Ordering::Greater => Color32::WHITE,
                    };

                    // Render breakpoing and instruction
                    let mut bk = cpu.breakpoint_at(*addr);
                    let resp = ui.checkbox(&mut bk, RichText::new(instr).color(color));

                    // Scroll to selected instruction or PC
                    if goto_addr == Some(*addr) {
                        ui.scroll_to_rect(resp.rect, Some(egui::Align::Center));
                    }

                    // Set/unset breakpoint
                    if resp.changed() {
                        if bk {
                            cpu.set_breakpoint(*addr);
                        } else {
                            cpu.clear_breakpoint(*addr);
                        }
                    }
                }
            });
    }
}
