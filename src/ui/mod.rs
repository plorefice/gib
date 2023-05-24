use std::{path::Path, thread};

use anyhow::Error;
use egui::Key;
use gib_core::{self, io::JoypadState};
use parking_lot::Mutex;
use sound::SoundEngine;
use state::Emulator;

mod sound;
mod state;
mod utils;
mod views;

const EMU_X_RES: usize = 160;
const EMU_Y_RES: usize = 144;

/// Emulator window width (in gaming mode)
const EMU_WIN_X_RES: f32 = (EMU_X_RES * 2) as f32;
/// Emulator window height (in gaming mode)
const EMU_WIN_Y_RES: f32 = (EMU_Y_RES * 2) as f32 + 24.;

/// Mapping between keycode and joypad button
const KEYMAP: [(Key, JoypadState); 8] = [
    (Key::ArrowUp, JoypadState::UP),
    (Key::ArrowDown, JoypadState::DOWN),
    (Key::ArrowLeft, JoypadState::LEFT),
    (Key::ArrowRight, JoypadState::RIGHT),
    (Key::Z, JoypadState::B),
    (Key::X, JoypadState::A),
    (Key::Backspace, JoypadState::SELECT),
    (Key::Enter, JoypadState::START),
];

use std::sync::Arc;

use crate::ui::views::WindowManager;

pub struct EmuUi {
    emu: Arc<Mutex<Emulator>>,
    vpu_buffer: Vec<u8>,
    vpu_texture: egui::TextureHandle,

    #[allow(dead_code)] // not actually used, but we can't drop it
    sound_engine: SoundEngine,

    window_manager: WindowManager,
    debug_mode: bool,
}

impl EmuUi {
    pub const WINDOW_SIZE: [f32; 2] = [EMU_WIN_X_RES, EMU_WIN_Y_RES];

    pub const DEVEL_WINDOW_SIZE: [f32; 2] = [1440., 720.];

    pub fn new(cc: &eframe::CreationContext<'_>, debug_mode: bool) -> Result<Self, Error> {
        // Create a sample channel that can hold up to 1024 samples.
        // At 44.1KHz, this is about 23ms worth of audio.
        let (source, sink) = gib_core::create_sound_channel(1024);

        // Start audio thread.
        // NOTE(windows): this needs to happen before the GUI is created, or the process
        // will throw an error regarding thread creation.
        let mut sound_engine = SoundEngine::new()?;
        sound_engine.start(sink)?;

        // Allocate a blank screen
        let vpu_buffer = vec![0xFFu8; EMU_X_RES * EMU_Y_RES * 4];
        let vpu_texture = cc.egui_ctx.load_texture(
            "gb-screen",
            egui::ColorImage::from_rgba_unmultiplied([EMU_X_RES, EMU_Y_RES], &vpu_buffer),
            egui::TextureOptions::NEAREST,
        );

        // Create and configure the emulator instance
        let mut emu = Emulator::default();
        emu.configure_audio_channel(source, sound_engine.get_sample_rate());

        Ok(EmuUi {
            emu: Arc::new(Mutex::new(emu)),
            vpu_buffer,
            vpu_texture,

            sound_engine,

            window_manager: Default::default(),
            debug_mode,
        })
    }

    /// Loads the ROM file and starts the emulation.
    pub fn load_rom<P: AsRef<Path>>(&mut self, rom: P) -> Result<(), Error> {
        let mut emu = self.emu.lock();

        emu.load_rom(rom)?;

        if self.debug_mode {
            emu.cpu_mut().allow_rollback_on_error(true);
        }

        emu.set_running();

        let emu = self.emu.clone();
        thread::spawn(move || loop {
            emu.lock().do_step();
        });

        Ok(())
    }

    fn update_emulation(&mut self, ctx: &egui::Context) {
        let mut emu = self.emu.lock();

        // Forward keypresses to the emulator
        for &(vk, js) in KEYMAP.iter() {
            if ctx.input(|i| i.key_down(vk)) {
                emu.gameboy_mut().press_key(js);
            } else {
                emu.gameboy_mut().release_key(js);
            }
        }

        // Enable/disable turbo mode
        emu.set_turbo(ctx.input(|i| i.key_down(Key::Space)));

        // Render to texture
        emu.gameboy().rasterize(&mut self.vpu_buffer[..]);

        // Update texture data
        ctx.tex_manager().write().set(
            self.vpu_texture.id(),
            egui::epaint::ImageDelta::full(
                egui::ColorImage::from_rgba_unmultiplied([EMU_X_RES, EMU_Y_RES], &self.vpu_buffer),
                egui::TextureOptions::NEAREST,
            ),
        );
    }
}

impl eframe::App for EmuUi {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.style(ctx);

        self.update_emulation(ctx);

        if self.debug_mode {
            self.debug_ui(ctx, frame);
        } else {
            self.game_ui(ctx, frame);
        }

        // The UI needs to be continuously refreshed, since the emulator updates in backgronud
        ctx.request_repaint();
    }
}

impl EmuUi {
    fn style(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.override_text_style = Some(egui::TextStyle::Monospace);
        ctx.set_style(style);
    }

    fn game_ui(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| self.emulation_menu_ui(ui, frame));

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.image(&self.vpu_texture, self.vpu_texture.size_vec2() * 2.)
            });
    }

    fn debug_ui(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| self.emulation_menu_ui(ui, frame));

        egui::CentralPanel::default().show(ctx, |ui| {
            self.window_manager.windows(ui.ctx(), &mut self.emu.lock());

            // Draw screen last for focus
            self.screen_ui(ui);
        });
    }

    fn screen_ui(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Screen")
            .default_pos([730., 30.])
            .show(ui.ctx(), |ui| {
                ui.image(&self.vpu_texture, self.vpu_texture.size_vec2());
            });
    }

    fn emulation_menu_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Emulator", |ui| {
                if ui.button("Load ROM...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.load_rom(path).unwrap();
                    }
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Save screen").clicked() {
                    std::fs::write("screen-dump.bin", &self.vpu_buffer[..]).ok();
                    ui.close_menu();
                }

                if ui.button("Reset").clicked() {
                    self.emu.lock().reset();
                    ui.close_menu();
                }

                if ui.button("Quit").clicked() {
                    frame.close();
                }
            })
        });
    }
}
