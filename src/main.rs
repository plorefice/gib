#![feature(pattern)]
#![feature(box_syntax)]
#![feature(duration_float)]
#![feature(range_contains)]
#![feature(try_from)]

mod gb;
mod ui;

fn main() {
    use std::env;

    let mut emu = ui::EmuUi::new(true);

    if let Some(ref fname) = env::args().nth(1) {
        emu.load_rom(fname).expect("error loading rom");
    }

    emu.run().expect("while running emulator");
}
