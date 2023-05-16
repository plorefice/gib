use std::path::PathBuf;

use clap::Parser;

mod ui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Use development UI
    #[arg(short, long)]
    devel: bool,

    /// ROM file to run
    rom: Option<PathBuf>,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let mut emu = ui::EmuUi::new(cli.devel).unwrap();

    if let Some(ref rom) = cli.rom {
        emu.load_rom(rom).expect("error loading rom");
    }

    emu.run().expect("while running emulator");
}
