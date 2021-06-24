use std::{
    cell::RefCell,
    collections::HashMap,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};

use crossbeam::queue::ArrayQueue;
use ctx::UiContext;
use failure::Error;
use gfx::{
    texture::{FilterMethod, SamplerInfo, WrapMode},
    Factory,
};
use gib_core::{self, io::JoypadState};
use glutin::event::VirtualKeyCode;
use imgui::{im_str, Condition, Image, MenuItem, StyleVar, Ui, Window, WindowFlags};
use sound::SoundEngine;
use state::EmuState;
use views::{
    DebuggerView, DisassemblyView, MemEditView, MemMapView, PeripheralView, View, WindowView,
};

mod ctx;
mod sound;
mod state;
mod utils;
mod views;

const EMU_X_RES: usize = 160;
const EMU_Y_RES: usize = 144;

/// Emulator window width (in gaming mode)
const EMU_WIN_X_RES: f64 = (EMU_X_RES * 2) as f64;
/// Emulator window height (in gaming mode)
const EMU_WIN_Y_RES: f64 = (EMU_Y_RES * 2) as f64 + 19.5;

/// Mapping between VirtualKey and joypad button
const KEYMAP: [(VirtualKeyCode, JoypadState); 8] = [
    (VirtualKeyCode::Up, JoypadState::UP),
    (VirtualKeyCode::Down, JoypadState::DOWN),
    (VirtualKeyCode::Left, JoypadState::LEFT),
    (VirtualKeyCode::Right, JoypadState::RIGHT),
    (VirtualKeyCode::Z, JoypadState::B),
    (VirtualKeyCode::X, JoypadState::A),
    (VirtualKeyCode::Back, JoypadState::SELECT),
    (VirtualKeyCode::Return, JoypadState::START),
];

pub struct GuiState {
    debug: bool,
    should_quit: bool,
    file_dialog: Option<utils::FileDialog>,
    views: HashMap<View, Box<dyn WindowView>>,
}

impl Default for GuiState {
    fn default() -> GuiState {
        GuiState {
            debug: false,
            should_quit: false,
            file_dialog: None,
            views: HashMap::new(),
        }
    }
}

use std::sync::{Arc, Mutex};

pub struct EmuUi {
    ctx: Rc<RefCell<UiContext>>,
    snd: SoundEngine,
    gui: GuiState,

    emu: Option<Arc<Mutex<EmuState>>>,
    vpu_buffer: Vec<u8>,
    vpu_texture: Option<imgui::TextureId>,

    snd_sink: Arc<ArrayQueue<i16>>,
}

impl EmuUi {
    pub fn new(debug: bool) -> Result<EmuUi, Error> {
        let gui = GuiState {
            debug,
            ..Default::default()
        };

        // Create a sample channel that can hold up to 1024 samples.
        // At 44.1KHz, this is about 23ms worth of audio.
        let sink = Arc::new(ArrayQueue::new(1024));

        // Start audio thread.
        // NOTE(windows): this needs to happen before the GUI is created, or the process
        // will throw an error regarding thread creation.
        let mut snd = SoundEngine::new()?;
        snd.start(sink.clone())?;

        // In debug mode, the interface is much more cluttered, so default to a bigger size
        let ctx = if debug {
            UiContext::new(1440.0, 720.0)
        } else {
            UiContext::new(EMU_WIN_X_RES, EMU_WIN_Y_RES)
        };

        Ok(EmuUi {
            ctx: Rc::from(RefCell::from(ctx)),
            snd,
            gui,

            emu: None,
            vpu_buffer: vec![0xFFu8; EMU_X_RES * EMU_Y_RES * 4],
            vpu_texture: None,

            snd_sink: sink,
        })
    }

    /// Loads the ROM file and starts the emulation.
    pub fn load_rom<P: AsRef<Path>>(&mut self, rom: P) -> Result<(), Error> {
        let emu = {
            let mut emu = EmuState::new(rom)?;
            emu.set_audio_sink(self.snd_sink.clone(), self.snd.get_sample_rate());
            emu.set_running();

            Arc::new(Mutex::new(emu))
        };

        if self.gui.debug {
            let views = &mut self.gui.views;

            // Start a new UI from scratch
            views.clear();

            views.insert(View::Disassembly, Box::new(DisassemblyView::new()));
            views.insert(View::Debugger, Box::new(DebuggerView::new()));
            views.insert(View::MemEditor, Box::new(MemEditView::new()));
            views.insert(View::Peripherals, Box::new(PeripheralView::new()));
        }

        // Spawn and start the emulation thread.
        //
        // TODO there really needs to be a way to stop this thread.
        {
            let emu = emu.clone();

            std::thread::spawn(move || {
                loop {
                    emu.lock().unwrap().do_step();

                    // After each step, we can sleep for a fraction of the audio buffer,
                    // or for much less if not in audio sync mode.
                    //
                    // TODO this is ugly, find a better paradigm to synchronize everything.
                    if !emu.lock().unwrap().turbo() {
                        std::thread::sleep(Duration::from_millis(5));
                    } else {
                        std::thread::sleep(Duration::from_micros(1));
                    }
                }
            });
        }

        self.emu = Some(emu);

        Ok(())
    }

    /// Run the emulator UI.
    ///
    /// This function loops until the window is closed or an error occurs.
    pub fn run(&mut self) -> Result<(), Error> {
        let mut last_frame = Instant::now();

        loop {
            let ctx = self.ctx.clone();
            let mut ctx = ctx.borrow_mut();

            // Compute time elapsed since last frame
            let frame_start = Instant::now();
            let delta = frame_start - last_frame;
            last_frame = frame_start;

            /*
             * Event handling phase
             */

            ctx.poll_events();

            if self.gui.should_quit || ctx.should_quit() {
                return Ok(());
            }

            /*
             * Emulator syncing phase
             */

            if let Some(ref mut emu) = self.emu {
                let emu = &mut emu.lock().unwrap();

                // Forward keypresses to the emulator
                for (vk, js) in KEYMAP.iter() {
                    if ctx.is_key_pressed(*vk) {
                        emu.gameboy_mut().press_key(*js);
                    } else {
                        emu.gameboy_mut().release_key(*js);
                    }
                }

                // Enable/disable turbo mode
                emu.set_turbo(ctx.is_key_pressed(VirtualKeyCode::Space));

                // TODO this really needs to be done only if some changes
                // have happened in the last interval.
                emu.gameboy().rasterize(&mut self.vpu_buffer[..]);
            }

            /*
             * Rendering phase
             */

            self.prepare_screen_texture(&mut *ctx);

            ctx.render(delta, |ui| {
                if self.gui.debug {
                    self.draw_debug_ui(delta.as_secs_f32(), ui)
                } else {
                    self.draw_game_ui(delta.as_secs_f32(), ui)
                }
            });

            // Pace the rendering thread
            std::thread::sleep(
                Duration::new(0, 1_000_000_000 / 60)
                    .checked_sub(Instant::now() - frame_start)
                    .unwrap_or_default(),
            );
        }
    }

    /// Creates a new texture displaying the currently emulated screen,
    /// ready to be presented during the next rendering step.
    fn prepare_screen_texture(&mut self, ctx: &mut UiContext) {
        let texture = ctx
            .factory
            .create_texture_immutable_u8::<gfx::format::Rgba8>(
                gfx::texture::Kind::D2(
                    EMU_X_RES as u16,
                    EMU_Y_RES as u16,
                    gfx::texture::AaMode::Single,
                ),
                gfx::texture::Mipmap::Provided,
                &[&self.vpu_buffer[..]],
            )
            .unwrap()
            .1;

        let sampler = ctx
            .factory
            .create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp));

        let texture = (texture, sampler);

        // If this is the first time rendering, insert the new texture, otherwise
        // replace an existing one.
        if let Some(ref vpu_texture) = self.vpu_texture {
            ctx.renderer.textures().replace(*vpu_texture, texture);
        } else {
            self.vpu_texture = Some(ctx.renderer.textures().insert(texture));
        }
    }

    /// Draws the gaming-mode interface, with just a simple menu bar
    /// and a fullscreen emulator screen view.
    fn draw_game_ui(&mut self, delta_s: f32, ui: &Ui) {
        self.draw_menu_bar(delta_s, ui);

        // Do not show window borders
        let style_vars = [
            StyleVar::WindowBorderSize(0.0),
            StyleVar::WindowRounding(0.0),
            StyleVar::WindowPadding([0.0, 0.0]),
        ];

        let win_x = EMU_WIN_X_RES as f32;
        let win_y = EMU_WIN_Y_RES as f32 - 18.0; // account for menu bar

        let style_tok = ui.push_style_vars(&style_vars);

        Window::new(im_str!("Screen"))
            .size([win_x, win_y], Condition::FirstUseEver)
            .position([0.0, 19.5], Condition::FirstUseEver)
            .flags(
                // Disable any window feature
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::NO_SCROLL_WITH_MOUSE,
            )
            .build(ui, || {
                // Display event, if any
                if let Some(ref emu) = self.emu {
                    if let Some(ref evt) = emu.lock().unwrap().last_event() {
                        ui.text_colored(utils::RED, evt.to_string());
                    }
                }

                if let Some(texture) = self.vpu_texture {
                    Image::new(texture, [win_x, win_y]).build(ui);
                }
            });

        style_tok.pop(ui);
    }

    /// Draws the debug-mode interface
    fn draw_debug_ui(&mut self, delta_s: f32, ui: &Ui) {
        self.draw_menu_bar(delta_s, ui);

        if self.emu.is_some() {
            self.draw_screen_window(ui);
        }

        if let Some(ref mut emu) = self.emu {
            let emu = &mut emu.lock().unwrap();
            self.gui.views.retain(|_, view| view.draw(ui, emu));
        }
    }

    fn draw_menu_bar(&mut self, delta_s: f32, ui: &Ui) {
        let emu_running = self.emu.is_some();

        self.draw_file_dialog(delta_s, ui);

        ui.main_menu_bar(|| {
            ui.menu(im_str!("Emulator"), true, || {
                if MenuItem::new(im_str!("Load ROM...")).build(ui) {
                    self.gui.file_dialog = Some(utils::FileDialog::new("Load ROM..."));
                }

                ui.separator();

                if MenuItem::new(im_str!("Save screen")).build(ui) {
                    std::fs::write("screen-dump.bin", &self.vpu_buffer[..]).unwrap();
                }

                if MenuItem::new(im_str!("Reset"))
                    .enabled(emu_running)
                    .build(ui)
                {
                    if let Some(ref mut emu) = self.emu {
                        emu.lock().unwrap().reset().expect("error during reset");
                    }
                }

                self.gui.should_quit = MenuItem::new(im_str!("Exit")).build(ui);
            });

            // Show debug-related menus in debug mode only
            if self.gui.debug {
                ui.menu(im_str!("Hardware"), true, || {
                    if MenuItem::new(im_str!("Memory Map"))
                        .enabled(emu_running)
                        .build(ui)
                    {
                        self.gui
                            .views
                            .entry(View::MemMap)
                            .or_insert_with(|| Box::new(MemMapView::new()));
                    }

                    if MenuItem::new(im_str!("Peripherals"))
                        .enabled(emu_running)
                        .build(ui)
                    {
                        self.gui
                            .views
                            .entry(View::Peripherals)
                            .or_insert_with(|| Box::new(PeripheralView::new()));
                    }
                });

                ui.menu(im_str!("Debugging"), true, || {
                    if MenuItem::new(im_str!("Debugger"))
                        .enabled(emu_running)
                        .build(ui)
                    {
                        self.gui
                            .views
                            .entry(View::Debugger)
                            .or_insert_with(|| Box::new(DebuggerView::new()));
                    }

                    if MenuItem::new(im_str!("Disassembler"))
                        .enabled(emu_running)
                        .build(ui)
                    {
                        self.gui
                            .views
                            .entry(View::Disassembly)
                            .or_insert_with(|| Box::new(DisassemblyView::new()));
                    }

                    if MenuItem::new(im_str!("Memory Editor"))
                        .enabled(emu_running)
                        .build(ui)
                    {
                        self.gui
                            .views
                            .entry(View::MemEditor)
                            .or_insert_with(|| Box::new(MemEditView::new()));
                    }
                })
            }
        });
    }

    fn draw_file_dialog(&mut self, delta_s: f32, ui: &Ui) {
        let mut fd_closed = false;
        let mut fd_chosen = None;

        if let Some(ref mut fd) = self.gui.file_dialog {
            fd.build(delta_s, ui, |res| {
                fd_closed = true;
                fd_chosen = res;
            });
        }
        if fd_closed {
            self.gui.file_dialog = None;
        }

        if let Some(ref rom_file) = fd_chosen {
            if let Err(evt) = self.load_rom(rom_file) {
                ui.popup_modal(im_str!("Error loading ROM")).build(|| {
                    ui.text(format!("{}", evt));
                });
                ui.open_popup(im_str!("Error loading ROM"));
            }
        }
    }

    fn draw_screen_window(&mut self, ui: &Ui) {
        Window::new(im_str!("Screen"))
            .size(
                [EMU_X_RES as f32 + 15.0, EMU_Y_RES as f32 + 40.0],
                Condition::FirstUseEver,
            )
            .position([745.0, 30.0], Condition::FirstUseEver)
            .resizable(false)
            .build(ui, || {
                if let Some(texture) = self.vpu_texture {
                    Image::new(texture, [EMU_X_RES as f32, EMU_Y_RES as f32]).build(ui);
                }
            });
    }
}
