use std::path::PathBuf;

use clap::Parser;

use crate::ui::EmuUi;

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

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let cli = Cli::parse();

    let options = eframe::NativeOptions {
        initial_window_size: Some(
            if cli.devel {
                EmuUi::DEVEL_WINDOW_SIZE
            } else {
                EmuUi::WINDOW_SIZE
            }
            .into(),
        ),
        maximized: cli.devel,
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "gib",
        options,
        Box::new(move |cc| match EmuUi::new(cc, cli.devel) {
            Ok(mut app) => {
                if let Some(rom) = cli.rom {
                    app.load_rom(rom).expect("failed to load rom");
                }
                Box::new(app)
            }
            Err(e) => panic!("{e}"),
        }),
    )
}
