mod debugger;
mod disassembly;
mod memedit;

pub use debugger::*;
pub use disassembly::*;
pub use memedit::*;

use super::utils;
use super::{EmuState, Immediate};

use imgui::Ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum View {
    Debugger,
    Disassembly,
    MemEditor,
}

pub trait WindowView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool;
}
