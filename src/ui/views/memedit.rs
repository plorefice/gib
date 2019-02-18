use gib_core::dbg;
use gib_core::mem::MemR;

use super::utils;
use super::EmuState;
use super::WindowView;

use imgui::{im_str, ImGuiCond, ImString, Ui};

/// View containing an hexadecimal dump of a selectable memory region.
pub struct MemEditView {
    section: dbg::MemoryType,
    content: Vec<ImString>,

    search_string: ImString,
    matched_lines: Vec<usize>,
    highlighted_line_id: Option<usize>,
    find_next: bool,
}

impl MemEditView {
    pub fn new() -> MemEditView {
        let max_bank_size = 0x4000 / 16;

        MemEditView {
            section: dbg::MemoryType::RomBank(0),
            content: Vec::with_capacity(max_bank_size),

            search_string: ImString::with_capacity(128),
            matched_lines: Vec::with_capacity(max_bank_size),
            highlighted_line_id: None,
            find_next: false,
        }
    }

    /// Refresh the view's content, by reading and rasterizing
    /// the whole memory section from scratch.
    fn refresh_memory(&mut self, state: &EmuState) {
        let bus = state.bus();

        let (mut ptr, end): (u32, u32) = {
            let mem_range = self.section.range();
            (
                u32::from(*mem_range.start()),
                u32::from(*mem_range.end()) + 1,
            )
        };

        self.content.clear();

        while ptr < end {
            let mut data = [0u8; 16];

            for addr in ptr..(ptr + 16).min(end) {
                match bus.read(addr as u16) {
                    Ok(b) => data[(addr - ptr) as usize] = b,
                    Err(e) => panic!("unexpected trace event during memory access: {}", e),
                };
            }

            // Eg: "0xFF00:  00 01 02 03 04 05  |...123|"
            let mut content = format!("{:04X}:  ", ptr);
            for d in data.iter() {
                content.push_str(&format!("{:02X} ", d));
            }
            content.push(' ');
            content.push_str(&utils::format_ascii(&data));

            self.content.push(content.into());

            ptr += 16;
        }
    }

    /// Finds the search pattern in the currently selected memory region.
    fn find_string(&mut self) {
        let pat = self.search_string.to_str();

        self.highlighted_line_id = None;

        if pat.is_empty() {
            self.matched_lines.clear();
        } else {
            self.matched_lines = self
                .content
                .iter()
                .enumerate()
                .filter_map(|(i, line)| {
                    if line.to_str().contains(pat) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();
        }
    }

    /// Cycles to the next occurrence of the search pattern in the search results.
    fn find_next(&mut self) {
        self.highlighted_line_id = match self.highlighted_line_id {
            Some(n) => Some((n + 1) % self.matched_lines.len()),
            None => {
                if self.matched_lines.is_empty() {
                    None
                } else {
                    Some(0)
                }
            }
        };
    }

    // Draw the memory change buttons and search input box on top of the memory viewer.
    fn draw_toolbar(&mut self, ui: &Ui, state: &EmuState) {
        use dbg::MemoryType::*;

        for (label, region) in [
            (im_str!("ROM00"), RomBank(0)),
            (im_str!("ROM01"), RomBank(1)),
            (im_str!("VRAM"), VideoRam),
            (im_str!("ERAM"), ExternalRam),
            (im_str!("WRAM00"), WorkRamBank(0)),
            (im_str!("WRAM01"), WorkRamBank(1)),
            (im_str!("HRAM"), HighRam),
        ]
        .iter()
        {
            if ui.button(label, (0.0, 0.0)) {
                self.section = *region;
                self.refresh_memory(state);
                self.find_string();
            }
            ui.same_line(0.0);
        }

        let (w, _) = ui.get_content_region_avail();

        // Check to see if the search string has changed,
        // and if it has, update the search results
        ui.with_item_width(w - 25.0, || {
            if ui.input_text(im_str!(""), &mut self.search_string).build() {
                self.find_string();
            }
        });
        ui.same_line(0.0);

        self.find_next = ui.button(im_str!(">"), (20.0, 0.0));
    }
}

impl WindowView for MemEditView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool {
        let mut open = true;

        // Refresh automatically the first time
        if self.content.is_empty() {
            self.refresh_memory(state);
        }

        ui.window(im_str!("Memory Editor"))
            .size((555.0, 400.0), ImGuiCond::FirstUseEver)
            .position((320.0, 280.0), ImGuiCond::FirstUseEver)
            .opened(&mut open)
            .build(|| {
                self.draw_toolbar(ui, state);

                ui.separator();

                let (_, h) = ui.get_content_region_avail();

                ui.child_frame(im_str!("memedit_listing"), (540.0, h))
                    .always_show_vertical_scroll_bar(true)
                    .show_borders(false)
                    .build(|| {
                        // Find and jump to the next result when requested
                        if self.find_next {
                            self.find_next = false;
                            self.find_next();

                            if let Some(n) = self.highlighted_line_id {
                                utils::scroll_to(ui, self.matched_lines[n], Some(h));
                            }
                        }

                        utils::list_clipper(ui, self.content.len(), |rng| {
                            for i in rng {
                                // Right now we are highlighting the entire line
                                if self.matched_lines.contains(&i) {
                                    ui.text_colored(utils::YELLOW, &self.content[i]);
                                } else {
                                    ui.text(&self.content[i]);
                                }
                            }
                        });
                    });
            });

        open
    }
}
