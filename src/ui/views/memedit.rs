use super::EmuState;
use super::WindowView;

use imgui::{ImGuiCond, Ui};

pub struct MemEditView;

impl MemEditView {
    pub fn new() -> MemEditView {
        MemEditView
    }
}

impl WindowView for MemEditView {
    fn draw(&mut self, ui: &Ui, _state: &mut EmuState) -> bool {
        let mut open = true;

        ui.window(im_str!("Memory Editor"))
            .size((390.0, 400.0), ImGuiCond::FirstUseEver)
            .position((320.0, 160.0), ImGuiCond::FirstUseEver)
            .opened(&mut open)
            .build(|| {});

        open
    }
}
