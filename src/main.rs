#![feature(pattern)]
#![feature(duration_float)]

#[macro_use]
extern crate imgui;
extern crate imgui_sys;

mod gb;
mod ui;

fn main() {
    use std::env;
    use std::fs;

    let mut emu = ui::EmuUi::new(true);

    if let Some(ref fname) = env::args().nth(1) {
        if let Ok(rom) = fs::read(fname) {
            if let Err(evt) = emu.load_rom(&rom[..]) {
                println!("Error loading ROM: {:?}", evt);
                std::process::exit(1);
            }
        };
    };

    emu.run();
}
