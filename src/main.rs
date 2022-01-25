// `im_str!` throws this pedantic error around in imgui 0.7
#![allow(clippy::transmute_ptr_to_ptr)]

use clap::{App, Arg};

mod ui;

fn main() {
    env_logger::init();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("devel")
                .short('d')
                .long("devel")
                .help("Open development GUI"),
        )
        .arg(Arg::new("ROM").help("ROM file to run").index(1))
        .get_matches();

    let mut emu = ui::EmuUi::new(matches.is_present("devel")).unwrap();

    if let Some(ref rom) = matches.value_of("ROM") {
        emu.load_rom(rom).expect("error loading rom");
    }

    emu.run().expect("while running emulator");
}
