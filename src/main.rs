mod ui;

fn main() {
    use clap::{App, Arg};

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("devel")
                .short("d")
                .long("devel")
                .help("Open development GUI"),
        )
        .arg(Arg::with_name("ROM").help("ROM file to run").index(1))
        .get_matches();

    let mut emu = ui::EmuUi::new(matches.is_present("devel")).unwrap();

    if let Some(ref rom) = matches.value_of("ROM") {
        emu.load_rom(rom).expect("error loading rom");
    }

    emu.run().expect("while running emulator");
}
