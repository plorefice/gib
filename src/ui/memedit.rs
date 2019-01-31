use super::utils;
use super::EmuState;

use imgui::{ImGuiCond, Ui};

pub struct MemoryEditor;

impl MemoryEditor {
    pub fn new() -> MemoryEditor {
        MemoryEditor
    }

    pub fn draw(&mut self, ui: &Ui, state: &mut EmuState) {
        ui.window(im_str!("Memory Editor"))
            .size((390.0, 400.0), ImGuiCond::FirstUseEver)
            .position((320.0, 160.0), ImGuiCond::FirstUseEver)
            .build(|| {});
    }
}
