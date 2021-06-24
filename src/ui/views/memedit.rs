use gib_core::dbg;
use gib_core::mem::MemR;
use imgui::ChildWindow;
use imgui::ListClipper;
use imgui::Window;

use super::utils;
use super::EmuState;
use super::WindowView;

use imgui::{im_str, Condition, ImString, Ui};

use std::ops::Range;

/// View containing an hexadecimal dump of a selectable memory region.
pub struct MemEditView {
    section: dbg::MemoryType,
    content: Vec<ImString>,

    search_string: ImString,
    matched_ranges: Vec<(usize, Range<usize>)>, // (line,range)
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
            matched_ranges: Vec::with_capacity(max_bank_size),
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
            self.matched_ranges.clear();
        } else {
            self.matched_ranges = self
                .content
                .iter()
                .enumerate()
                .filter_map(|(i, line)| {
                    // TODO right now, only the first match of each line is found
                    line.to_str()
                        .find(pat)
                        .map(|start| (i, start..start + pat.len()))
                })
                .collect();
        }
    }

    /// Cycles to the next occurrence of the search pattern in the search results.
    fn find_next(&mut self) {
        self.highlighted_line_id = match self.highlighted_line_id {
            Some(n) => Some((n + 1) % self.matched_ranges.len()),
            None => {
                if self.matched_ranges.is_empty() {
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
            if ui.button(label, [0.0, 0.0]) {
                self.section = *region;
                self.refresh_memory(state);
                self.find_string();
            }
            ui.same_line(0.0);
        }

        let [w, _] = ui.content_region_avail();

        // Check to see if the search string has changed,
        // and if it has, update the search results
        let width_tok = ui.push_item_width(w - 25.);

        if ui.input_text(im_str!(""), &mut self.search_string).build() {
            self.find_string();
        }

        width_tok.pop(ui);

        ui.same_line(0.0);

        self.find_next = ui.button(im_str!(">"), [20.0, 0.0]);
    }
}

impl WindowView for MemEditView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool {
        let mut open = true;

        // Refresh automatically the first time
        if self.content.is_empty() {
            self.refresh_memory(state);
        }

        Window::new(im_str!("Memory Editor"))
            .size([555.0, 400.0], Condition::FirstUseEver)
            .position([320.0, 280.0], Condition::FirstUseEver)
            .opened(&mut open)
            .build(ui, || {
                self.draw_toolbar(ui, state);

                ui.separator();

                let [_, h] = ui.content_region_avail();

                ChildWindow::new("memedit_listing")
                    .size([540.0, h])
                    .always_vertical_scrollbar(true)
                    .border(false)
                    .build(ui, || {
                        // Find and jump to the next result when requested
                        if self.find_next {
                            self.find_next = false;
                            self.find_next();

                            if let Some(n) = self.highlighted_line_id {
                                utils::scroll_to(ui, self.matched_ranges[n].0, Some(h));
                            }
                        }

                        let mut clipper = ListClipper::new(self.content.len() as i32)
                            .items_height(ui.text_line_height_with_spacing())
                            .begin(ui);

                        while clipper.step() {
                            for i in clipper.display_start()..clipper.display_end() {
                                let i = i as usize;

                                let highlight = self.matched_ranges.iter().find(|(n, _)| *n == i);

                                if let Some((_, rng)) = highlight {
                                    let s = self.content[i].to_str();

                                    ui.text(&s[..rng.start]);
                                    ui.same_line_with_spacing(0.0, 0.0);
                                    ui.text_colored(utils::YELLOW, im_str!("{}", &s[rng.clone()]));
                                    ui.same_line_with_spacing(0.0, 0.0);
                                    ui.text(&s[rng.end..]);
                                } else {
                                    ui.text(&self.content[i]);
                                }
                            }
                        }
                    });
            });

        open
    }
}
