use super::gb::dbg::MemoryType;
use super::utils;
use super::{EmuState, WindowView};

use imgui::{im_str, ImGuiCol, ImGuiCond, ImStr, ImString, Ui};

pub struct MemMapView(Vec<(MemoryType, ImString)>);

impl MemMapView {
    pub fn new() -> MemMapView {
        let mut map = vec![];

        for mt in MemoryType::default().iter() {
            let r = mt.range();
            map.push((
                mt,
                ImString::new(format!("  {:04X}-{:04X}    {}\n", r.start(), r.end(), mt)),
            ));
        }
        MemMapView(map)
    }
}

impl WindowView for MemMapView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool {
        let mut open = true;

        ui.window(im_str!("Memory Map"))
            .size((225.0, 290.0), ImGuiCond::FirstUseEver)
            .position((720.0, 225.0), ImGuiCond::FirstUseEver)
            .opened(&mut open)
            .build(|| {
                let pc = state.cpu().pc;

                ui.spacing();
                for (mt, s) in self.0.iter() {
                    let c = if MemoryType::at(pc) == *mt {
                        utils::GREEN
                    } else {
                        utils::WHITE
                    };

                    ui.with_color_var(ImGuiCol::Text, c, || {
                        ui.text(ImStr::new(s));
                        ui.spacing();
                    });
                }
            });

        open
    }
}
