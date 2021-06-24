use gib_core::dbg::MemoryType;
use imgui::{im_str, Condition, ImString, Ui, Window};

use crate::ui::{state::EmuState, utils};

use super::WindowView;

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

        Window::new(im_str!("Memory Map"))
            .size([225.0, 290.0], Condition::FirstUseEver)
            .position([720.0, 225.0], Condition::FirstUseEver)
            .opened(&mut open)
            .build(ui, || {
                let pc = state.cpu().pc;

                ui.spacing();
                for (mt, s) in self.0.iter() {
                    let c = if MemoryType::at(pc) == *mt {
                        utils::GREEN
                    } else {
                        utils::WHITE
                    };

                    ui.text_colored(c, s);
                    ui.spacing();
                }
            });

        open
    }
}
