mod debugger;
mod disassembly;
mod memedit;
mod memmap;

pub use debugger::*;
pub use disassembly::*;
pub use memedit::*;
pub use memmap::*;

use super::dbg;
use super::utils;
use super::{EmuState, Immediate};

use imgui::Ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum View {
    Debugger,
    Disassembly,
    MemEditor,
    MemMap,
}

pub trait WindowView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool;
}
