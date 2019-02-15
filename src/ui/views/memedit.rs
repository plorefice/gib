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
}

impl MemEditView {
    pub fn new() -> MemEditView {
        MemEditView {
            section: dbg::MemoryType::RomBank(0),
            content: Vec::with_capacity(0x4000 / 16),
        }
    }

    /// Refresh the view's content, by reading and rasterizing
    /// the whole memory section from scratch.
    fn refresh_memory(&mut self, state: &EmuState) {
        let bus = state.bus();

        let mem_range = self.section.range();
        let mut ptr = *mem_range.start();

        self.content.clear();

        while ptr < *mem_range.end() {
            let mut data = [0u8; 16];

            for addr in ptr..ptr + 16 {
                match bus.read(addr) {
                    Ok(b) => data[usize::from(addr - ptr)] = b,
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

    // Draw buttons on top of the memory viewer to change memory localtion
    fn draw_toolbar(&mut self, ui: &Ui, state: &EmuState) {
        use dbg::MemoryType::*;

        for (i, (label, region)) in [
            (im_str!("ROM00"), RomBank(0)),
            (im_str!("ROM01"), RomBank(1)),
            (im_str!("VRAM"), VideoRam),
            (im_str!("ERAM"), ExternalRam),
            (im_str!("WRAM00"), WorkRamBank(0)),
            (im_str!("WRAM01"), WorkRamBank(1)),
            (im_str!("HRAM"), HighRam),
        ]
        .iter()
        .enumerate()
        {
            if i != 0 {
                ui.same_line(0.0);
            }

            if ui.button(label, (0.0, 0.0)) {
                self.section = *region;
                self.refresh_memory(state);
            }
        }
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
                        utils::list_clipper(ui, self.content.len(), |rng| {
                            for i in rng {
                                ui.text(&self.content[i]);
                            }
                        });
                    });
            });

        open
    }
}
