use std::{fmt::Write, ops::Range};

use gib_core::{dbg, mem::MemR};

use crate::ui::{state::EmuState, utils};

/// View containing an hexadecimal dump of a selectable memory region.
pub struct MemoryView {
    section: dbg::MemoryType,
    buffer: MemoryBuffer,

    search_string: String,
    matched_ranges: Vec<Range<usize>>,
    highlighted_line_id: Option<usize>,
}

impl Default for MemoryView {
    fn default() -> Self {
        let max_bank_size = 0x4000 / 16;

        MemoryView {
            section: dbg::MemoryType::RomBank(0),
            buffer: MemoryBuffer::with_capacity(256 * max_bank_size),

            search_string: String::with_capacity(128),
            matched_ranges: Vec::with_capacity(max_bank_size),
            highlighted_line_id: None,
        }
    }
}

impl super::Window for MemoryView {
    fn name(&self) -> &'static str {
        "Memory Editor"
    }

    fn show(&mut self, ctx: &egui::Context, state: &mut EmuState, open: &mut bool) {
        egui::Window::new(self.name())
            .default_pos([330.0, 220.0])
            .default_size([565.0, 460.0])
            .open(open)
            .show(ctx, |ui| {
                use super::View;
                self.ui(ui, state);
            });
    }
}

impl super::View for MemoryView {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut EmuState) {
        // Refresh automatically the first time
        if self.buffer.as_ref().is_empty() {
            self.buffer.refresh(self.section, state);
        }

        let find_next = self.toolbar_ui(ui, state);
        if find_next {
            self.find_next_match();
        }

        ui.separator();

        let mut layouter = |ui: &egui::Ui, s: &str, wrap_width: f32| {
            use egui::{
                text::{LayoutJob, TextFormat},
                Color32, FontId,
            };

            const FONT: FontId = FontId::monospace(12.);
            let simple = TextFormat::simple(FONT, Color32::WHITE);
            let highlight = TextFormat {
                font_id: FONT,
                color: Color32::BLACK,
                background: Color32::YELLOW,
                ..Default::default()
            };

            let mut layout_job = LayoutJob::default();

            if self.matched_ranges.is_empty() {
                layout_job.append(s, 0., simple);
            } else {
                let mut cursor_pos = 0;
                for rng in &self.matched_ranges {
                    layout_job.append(&s[cursor_pos..rng.start], 0., simple.clone());
                    layout_job.append(&s[rng.clone()], 0., highlight.clone());
                    cursor_pos = rng.end;
                }
                layout_job.append(&s[cursor_pos..], 0., simple);
            }

            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let rect = ui
                    .add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(<MemoryBuffer as AsMut<String>>::as_mut(
                            &mut self.buffer,
                        ))
                        .layouter(&mut layouter)
                        .interactive(false)
                        .frame(false),
                    )
                    .rect;

                // Scroll to the next occurrence
                if let Some(i) = find_next.then_some(self.highlighted_line_id).flatten() {
                    let rng = &self.matched_ranges[i];
                    let line = rng.start / self.buffer.line_len;
                    let line_height = rect.height() / self.buffer.lines as f32;

                    let y_start = rect.y_range().start() + line as f32 * line_height;

                    ui.scroll_to_rect(
                        egui::Rect::from_x_y_ranges(
                            rect.x_range(),
                            y_start..=y_start + line_height,
                        ),
                        Some(egui::Align::Center),
                    );
                }
            });
    }
}

impl MemoryView {
    /// Finds the search pattern in the currently selected memory region.
    fn find_search_pattern(&mut self) {
        let pat = &self.search_string;

        self.highlighted_line_id = None;

        if pat.is_empty() {
            self.matched_ranges.clear();
        } else {
            self.matched_ranges = self
                .buffer
                .as_ref()
                .match_indices(pat)
                .map(|(start, _)| start..start + pat.len())
                .collect();
        }
    }

    /// Cycles to the next occurrence of the search pattern in the search results.
    fn find_next_match(&mut self) {
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

    /// Draws the memory change buttons and search input box on top of the memory viewer.
    ///
    /// Returns whether the "Find next match" button was pressed.
    fn toolbar_ui(&mut self, ui: &mut egui::Ui, state: &EmuState) -> bool {
        use dbg::MemoryType::*;

        ui.horizontal(|ui| {
            for (label, region) in [
                ("ROM00", RomBank(0)),
                ("ROM01", RomBank(1)),
                ("VRAM", VideoRam),
                ("ERAM", ExternalRam),
                ("WRAM00", WorkRamBank(0)),
                ("WRAM01", WorkRamBank(1)),
                ("HRAM", HighRam),
            ]
            .into_iter()
            {
                if ui.button(label).clicked() {
                    self.section = region;
                    self.buffer.refresh(self.section, state);
                    self.find_search_pattern();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let find_next = ui.button(">").clicked();

                let output = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut self.search_string),
                );
                if output.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.find_search_pattern();
                }

                find_next
            })
            .inner
        })
        .inner
    }
}

struct MemoryBuffer {
    contents: String,
    line_len: usize,
    lines: usize,
}

impl AsRef<str> for MemoryBuffer {
    fn as_ref(&self) -> &str {
        &self.contents
    }
}

impl AsMut<str> for MemoryBuffer {
    fn as_mut(&mut self) -> &mut str {
        &mut self.contents
    }
}

impl AsMut<String> for MemoryBuffer {
    fn as_mut(&mut self) -> &mut String {
        &mut self.contents
    }
}

impl MemoryBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            contents: String::with_capacity(capacity),
            line_len: 0,
            lines: 0,
        }
    }

    /// Rebuilds the buffer contents, by reading and rasterizing the whole memory section.
    fn refresh(&mut self, section: dbg::MemoryType, state: &EmuState) {
        let bus = state.bus();

        let (mut ptr, end): (u32, u32) = {
            let mem_range = section.range();
            (
                u32::from(*mem_range.start()),
                u32::from(*mem_range.end()) + 1,
            )
        };

        self.contents.clear();

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
                write!(content, "{d:02X} ").unwrap();
            }
            content.push(' ');
            content.push_str(&utils::format_ascii(&data));
            content.push('\n');

            self.contents.push_str(&content);

            self.line_len = self.line_len.max(content.len());
            self.lines += 1;

            ptr += 16;
        }
    }
}
