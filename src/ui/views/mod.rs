pub use debugger::*;
pub use disassembly::*;
use imgui::Ui;
pub use memedit::*;
pub use memmap::*;
pub use peripherals::*;

use super::state::EmuState;

mod debugger;
mod disassembly;
mod memedit;
mod memmap;
mod peripherals;

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
