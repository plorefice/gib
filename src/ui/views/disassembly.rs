use super::gb::{cpu::Immediate, dbg};
use super::utils;
use super::{EmuState, WindowView};

use std::collections::BTreeMap;

use imgui::{im_str, ImGuiCol, ImGuiCond, ImStr, ImString, StyleVar, Ui};

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
        let (_, h) = ui.get_content_region_avail();

        if !self.disasm.contains_key(&dest) {
            self.realign_disasm(state, dest);
        }

        for (i, addr) in self.disasm.keys().enumerate() {
            if *addr == dest {
                unsafe {
                    imgui_sys::igSetScrollY(
                        ui.get_text_line_height_with_spacing() * i as f32 - h / 3.0,
                    );
                }
                break;
            }
        }
    }

    fn draw_goto_bar(&mut self, ui: &Ui) -> (bool, bool) {
        let goto_pc;
        let goto_addr;

        utils::input_addr(ui, "", &mut self.goto_addr, true);
        ui.same_line(0.0);

        goto_addr = ui.button(im_str!("Goto"), (0.0, 0.0));
        ui.same_line(0.0);

        goto_pc = ui.button(im_str!("Goto PC"), (0.0, 0.0));
        ui.same_line(0.0);

        ui.checkbox(im_str!("Follow"), &mut self.follow_pc);

        (goto_addr, goto_pc)
    }

    fn draw_disasm_view(&mut self, ui: &Ui, state: &mut EmuState, goto_addr: bool, goto_pc: bool) {
        let pc = state.cpu().pc;

        let (_, h) = ui.get_content_region_avail();

        ui.child_frame(im_str!("listing"), (285.0, h))
            .always_show_vertical_scroll_bar(true)
            .show_borders(false)
            .build(|| {
                if self.follow_pc || goto_pc {
                    self.goto(ui, state, pc);
                } else if goto_addr && self.goto_addr.is_some() {
                    self.goto(ui, state, self.goto_addr.unwrap());
                }

                // Only render currently visible instructions
                utils::list_clipper(ui, self.disasm.len(), |range| {
                    let instrs = self
                        .disasm
                        .iter_mut()
                        .skip(range.start)
                        .take(range.end - range.start);

                    let cpu = state.cpu_mut();

                    let style = &[StyleVar::FrameRounding(15.0)];

                    for (addr, instr) in instrs {
                        let color = &[(
                            ImGuiCol::Text,
                            if *addr < pc {
                                utils::DARK_GREY
                            } else if *addr == pc {
                                utils::GREEN
                            } else {
                                utils::WHITE
                            },
                        )];

                        // Render breakpoing and instruction
                        ui.with_style_and_color_vars(style, color, || {
                            let mut bk = cpu.breakpoint_at(*addr);

                            if ui.checkbox(ImStr::new(instr), &mut bk) {
                                if bk {
                                    cpu.set_breakpoint(*addr);
                                } else {
                                    cpu.clear_breakpoint(*addr);
                                }
                            }
                        });
                    }
                });
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

        ui.window(ImStr::new(&ImString::from(format!(
            "{} disassembly",
            self.section
        ))))
        .size((300.0, 650.0), ImGuiCond::FirstUseEver)
        .position((10.0, 30.0), ImGuiCond::FirstUseEver)
        .opened(&mut open)
        .build(|| {
            let (goto_addr, goto_pc) = self.draw_goto_bar(ui);

            ui.separator();

            self.draw_disasm_view(ui, state, goto_addr, goto_pc);
        });

        open
    }
}
