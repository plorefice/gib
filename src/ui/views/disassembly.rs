use std::{cmp::Ordering, collections::BTreeMap};

use gib_core::{cpu::Immediate, dbg};
use imgui::{Condition, ImString, ListClipper, StyleColor, StyleVar, Ui};

use crate::ui::{state::EmuState, utils};

use super::WindowView;

pub struct DisassemblyView {
    section: dbg::MemoryType,
    disasm: BTreeMap<u16, ImString>,
    follow_pc: bool,
    goto_addr: Option<u16>,
}

impl DisassemblyView {
    pub fn new() -> DisassemblyView {
        DisassemblyView {
            section: dbg::MemoryType::RomBank(0),
            disasm: BTreeMap::new(),
            follow_pc: false,
            goto_addr: Some(0),
        }
    }

    /// If there is alread an instruction decoded at address `from`, do nothing.
    /// Otherwise, fetch the instruction at from, invalidate all the overlapping
    /// instructions and update the disassembly. Do this until it's aligned again.
    /// If `from` is outside the current memory space, swap it and reload disasm.
    fn realign_disasm(&mut self, state: &EmuState, mut from: u16) {
        let cpu = state.cpu();
        let bus = state.bus();

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
                ImString::from(format!(
                    "{:04X}:  {:02X} {:5}    {}",
                    from,
                    instr.opcode,
                    match instr.imm {
                        Some(Immediate::Imm8(d8)) => format!("{:02X}", d8),
                        Some(Immediate::Imm16(d16)) => format!("{:04X}", d16),
                        None => String::new(),
                    },
                    instr.mnemonic
                )),
            );
            from = next;
        }
    }

    /// Scroll disassembly view to the desired address.
    fn goto(&mut self, ui: &Ui, state: &EmuState, dest: u16) {
        let [_, h] = ui.content_region_avail();

        if !self.disasm.contains_key(&dest) {
            self.realign_disasm(state, dest);
        }

        for (i, addr) in self.disasm.keys().enumerate() {
            if *addr == dest {
                // Some(h * 0.6) is to compensate for the fact that a disassembly line
                // is a bit taller that a line of text, due to the radio button.
                utils::scroll_to(ui, i, Some(h * 0.6));
                break;
            }
        }
    }

    fn draw_goto_bar(&mut self, ui: &Ui) -> (bool, bool) {
        utils::input_addr(ui, "disasm_goto", &mut self.goto_addr, true);
        ui.same_line();

        let goto_addr = ui.button("Goto");
        ui.same_line();

        let goto_pc = ui.button("Goto PC");
        ui.same_line();

        ui.checkbox("Follow", &mut self.follow_pc);

        (goto_addr, goto_pc)
    }

    fn draw_disasm_view(&mut self, ui: &Ui, state: &mut EmuState, goto_addr: bool, goto_pc: bool) {
        let pc = state.cpu().pc;

        let [_, h] = ui.content_region_avail();

        ui.child_window("listing")
            .size([285.0, h])
            .always_vertical_scrollbar(true)
            .border(false)
            .build(|| {
                if self.follow_pc || goto_pc {
                    self.goto(ui, state, pc);
                } else if goto_addr && self.goto_addr.is_some() {
                    self.goto(ui, state, self.goto_addr.unwrap());
                }

                // Only render currently visible instructions
                let mut clipper = ListClipper::new(self.disasm.len() as i32)
                    .items_height(ui.text_line_height_with_spacing())
                    .begin(ui);

                while clipper.step() {
                    let instrs = self
                        .disasm
                        .iter_mut()
                        .skip(clipper.display_start() as usize)
                        .take((clipper.display_end() - clipper.display_start()) as usize);

                    let cpu = state.cpu_mut();

                    let _style_tok = ui.push_style_var(StyleVar::FrameRounding(15.0));

                    for (addr, instr) in instrs {
                        let color = match addr.cmp(&pc) {
                            Ordering::Less => utils::DARK_GREY,
                            Ordering::Equal => utils::GREEN,
                            Ordering::Greater => utils::WHITE,
                        };

                        // Render breakpoing and instruction
                        let _color_tok = ui.push_style_color(StyleColor::Text, color);

                        let mut bk = cpu.breakpoint_at(*addr);
                        if ui.checkbox(instr, &mut bk) {
                            if bk {
                                cpu.set_breakpoint(*addr);
                            } else {
                                cpu.clear_breakpoint(*addr);
                            }
                        }
                    }
                }
            });
    }
}

impl WindowView for DisassemblyView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool {
        let mut open = true;

        // 99.9% of the time this does nothing, so it's cool
        // to have it called every draw loop.
        let pc = state.cpu().pc;
        self.realign_disasm(state, pc);

        ui.window("Disassembly")
            .size([300.0, 650.0], Condition::FirstUseEver)
            .position([10.0, 30.0], Condition::FirstUseEver)
            .opened(&mut open)
            .build(|| {
                let (goto_addr, goto_pc) = self.draw_goto_bar(ui);

                ui.separator();

                self.draw_disasm_view(ui, state, goto_addr, goto_pc);
            });

        open
    }
}
