mod debugger;
mod disassembly;
mod memedit;
mod memmap;
mod peripherals;

pub use debugger::*;
pub use disassembly::*;
pub use memedit::*;
pub use memmap::*;
pub use peripherals::*;

use super::utils;
use super::EmuState;

use imgui::Ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum View {
    Debugger,
    Disassembly,
    MemEditor,
    MemMap,
    Peripherals,
}

pub trait WindowView {
    fn draw(&mut self, ui: &Ui, state: &mut EmuState) -> bool;
}
