use std::collections::BTreeSet;

use crate::ui::state::Emulator;

pub mod debugger;
pub mod disassembly;
pub mod memedit;
pub mod memmap;
pub mod peripherals;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut Emulator);
}

pub trait Window {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, state: &mut Emulator, open: &mut bool);
}

pub struct WindowManager {
    windows: Vec<Box<dyn Window>>,
    open: BTreeSet<String>,
}

impl Default for WindowManager {
    fn default() -> Self {
        let windows: Vec<Box<dyn Window>> = vec![
            Box::<debugger::Debugger>::default(),
            Box::<disassembly::Disassembly>::default(),
            Box::<memedit::MemoryView>::default(),
            Box::<memmap::MemoryMap>::default(),
            Box::<peripherals::Peripherals>::default(),
        ];
        let open = BTreeSet::from_iter(windows.iter().map(|w| w.name().to_owned()));

        Self { windows, open }
    }
}

impl WindowManager {
    pub fn windows(&mut self, ctx: &egui::Context, state: &mut Emulator) {
        let Self { windows, open } = self;
        for window in windows {
            let name = window.name();
            let mut is_open = open.contains(name);
            window.show(ctx, state, &mut is_open);

            if is_open {
                if !open.contains(name) {
                    open.insert(name.to_owned());
                }
            } else {
                open.remove(name);
            }
        }
    }
}
