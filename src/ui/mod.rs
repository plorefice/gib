use gib_core::{self, io::JoypadState};

mod ctx;
mod sound;
mod state;
mod utils;
mod views;

use ctx::UiContext;
use sound::SoundEngine;
use state::EmuState;
use views::{
    DebuggerView, DisassemblyView, MemEditView, MemMapView, PeripheralView, View, WindowView,
};

use failure::Error;

use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use gfx_core::factory::Factory;
use glutin::VirtualKeyCode as Key;

use imgui::{im_str, ImGuiCond, Ui};

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

const EMU_X_RES: usize = 160;
const EMU_Y_RES: usize = 144;

/// Emulator window width (in gaming mode)
const EMU_WIN_X_RES: f64 = (EMU_X_RES * 2) as f64;
/// Emulator window height (in gaming mode)
const EMU_WIN_Y_RES: f64 = (EMU_Y_RES * 2) as f64 + 19.5;

/// Duration of a Game Boy frame
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// Mapping between VirtualKey and joypad button
const KEYMAP: [(Key, JoypadState); 8] = [
    (Key::Up, JoypadState::UP),
    (Key::Down, JoypadState::DOWN),
    (Key::Left, JoypadState::LEFT),
    (Key::Right, JoypadState::RIGHT),
    (Key::Z, JoypadState::B),
    (Key::X, JoypadState::A),
    (Key::Back, JoypadState::SELECT),
    (Key::Return, JoypadState::START),
];

pub struct GuiState {
    debug: bool,
    should_quit: bool,
    file_dialog: Option<utils::FileDialog>,
    views: HashMap<View, Box<WindowView>>,
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

pub struct EmuUi {
    ctx: Rc<RefCell<UiContext>>,
    snd: SoundEngine,
    gui: GuiState,

    emu: Option<EmuState>,
    vpu_buffer: Vec<u8>,
    vpu_texture: imgui::ImTexture,
}

impl EmuUi {
    pub fn new(debug: bool) -> EmuUi {
        let mut gui = GuiState::default();
        gui.debug = debug;

        // In debug mode, the interface is much more cluttered, so default to a bigger size
        let mut ctx = if debug {
            UiContext::new(1440.0, 720.0)
        } else {
            UiContext::new(EMU_WIN_X_RES, EMU_WIN_Y_RES)
        };

        let vpu_buffer = vec![0xFFu8; EMU_X_RES * EMU_Y_RES * 4];
        let texture = ctx
            .factory
            .create_texture_immutable_u8::<gfx::format::Rgba8>(
                gfx::texture::Kind::D2(
                    EMU_X_RES as u16,
                    EMU_Y_RES as u16,
                    gfx::texture::AaMode::Single,
                ),
                gfx::texture::Mipmap::Provided,
                &[&vpu_buffer[..]],
            )
            .unwrap()
            .1;

        let sampler = ctx
            .factory
            .create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp));

        let vpu_texture = ctx.renderer.textures().insert((texture, sampler));

        EmuUi {
            ctx: Rc::from(RefCell::from(ctx)),
            snd: SoundEngine::start().unwrap(),
            gui,

            emu: None,
            vpu_buffer,
            vpu_texture,
        }
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, rom: P) -> Result<(), Error> {
        self.emu = Some(EmuState::new(rom)?);

        if self.gui.debug {
            let views = &mut self.gui.views;

            // Start a new UI from scratch
            views.clear();

            views.insert(View::Disassembly, box DisassemblyView::new());
            views.insert(View::Debugger, box DebuggerView::new());
            views.insert(View::MemEditor, box MemEditView::new());
            views.insert(View::MemMap, box MemMapView::new());
            views.insert(View::Peripherals, box PeripheralView::new());
        }

        if let Some(ref mut emu) = self.emu {
            emu.set_running();
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut last_frame = Instant::now();
        let mut render_duration = Duration::new(0, 0);

        loop {
            let ctx = self.ctx.clone();
            let mut ctx = ctx.borrow_mut();

            ctx.poll_events();

            if self.gui.should_quit || ctx.should_quit() {
                return Ok(());
            }

            // Compute time elapsed since last frame
            let frame_start = Instant::now();
            let delta = frame_start - last_frame;
            let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
            last_frame = frame_start;

            if let Some(ref mut emu) = self.emu {
                // Handle keypresses
                for (vk, js) in KEYMAP.iter() {
                    if ctx.is_key_pressed(*vk) {
                        emu.gameboy_mut().press_key(*js);
                    } else {
                        emu.gameboy_mut().release_key(*js);
                    }
                }

                if ctx.is_key_pressed(Key::Space) {
                    // If the TURBO key is pressed, emulates as many V-blanks as possible
                    // without dropping _too much_ below 60 FPS, accounting for render time
                    emu.run_for(FRAME_DURATION - render_duration, &mut self.vpu_buffer[..]);
                } else {
                    emu.do_step(&mut self.vpu_buffer[..]);
                }

                // Push sound update
                self.snd
                    .push_new_sample(emu.gameboy().get_sound_output())
                    .unwrap();
            }

            // Measure how long it takes to render a frame. This is used in TURBO mode.
            render_duration = utils::measure_exec_time(|| {
                let new_screen = ctx
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

                ctx.renderer
                    .textures()
                    .replace(self.vpu_texture, (new_screen, sampler));

                ctx.render(delta_s, |ui| {
                    if self.gui.debug {
                        self.draw_debug_ui(delta_s, ui)
                    } else {
                        self.draw_game_ui(delta_s, ui)
                    }
                });
            });

            // Pace the emulation to the correct frame duration
            thread::sleep(
                FRAME_DURATION
                    .checked_sub(Instant::now() - frame_start)
                    .unwrap_or_default(),
            );
        }
    }

    /// Draws the gaming-mode interface, with just a simple menu bar
    /// and a fullscreen emulator screen view.
    fn draw_game_ui(&mut self, delta_s: f32, ui: &Ui) {
        use imgui::{ImGuiCol, ImGuiWindowFlags, ImVec2, StyleVar};

        self.draw_menu_bar(delta_s, ui);

        // Do not show window borders
        let style_vars = [
            StyleVar::WindowBorderSize(0.0),
            StyleVar::WindowRounding(0.0),
            StyleVar::WindowPadding(ImVec2::new(0.0, 0.0)),
        ];

        let win_x = EMU_WIN_X_RES as f32;
        let win_y = EMU_WIN_Y_RES as f32 - 18.0; // account for menu bar

        ui.with_style_vars(&style_vars, || {
            ui.window(im_str!("Screen"))
                .size((win_x, win_y), ImGuiCond::FirstUseEver)
                .position((0.0, 19.5), ImGuiCond::FirstUseEver)
                .flags(
                    // Disable any window feature
                    ImGuiWindowFlags::NoTitleBar
                        | ImGuiWindowFlags::NoResize
                        | ImGuiWindowFlags::NoMove
                        | ImGuiWindowFlags::NoScrollbar
                        | ImGuiWindowFlags::NoScrollWithMouse,
                )
                .build(|| {
                    // Display event, if any
                    if let Some(ref emu) = self.emu {
                        if let Some(ref evt) = emu.last_event() {
                            ui.with_color_var(ImGuiCol::Text, utils::RED, || {
                                ui.text(&format!("{}", evt))
                            });
                        }
                    }

                    ui.image(self.vpu_texture, (win_x, win_y)).build();
                });
        });
    }

    /// Draws the debug-mode interface
    fn draw_debug_ui(&mut self, delta_s: f32, ui: &Ui) {
        self.draw_menu_bar(delta_s, ui);

        if self.emu.is_some() {
            self.draw_screen_window(ui);
        }

        if let Some(ref mut emu) = self.emu {
            self.gui.views.retain(|_, view| view.draw(ui, emu));
        }
    }

    fn draw_menu_bar(&mut self, delta_s: f32, ui: &Ui) {
        let emu_running = self.emu.is_some();

        self.draw_file_dialog(delta_s, ui);

        ui.main_menu_bar(|| {
            ui.menu(im_str!("Emulator")).build(|| {
                if ui.menu_item(im_str!("Load ROM...")).build() {
                    self.gui.file_dialog = Some(utils::FileDialog::new("Load ROM..."));
                }

                ui.separator();

                if ui.menu_item(im_str!("Save screen")).build() {
                    std::fs::write("screen-dump.bin", &self.vpu_buffer[..]).unwrap();
                }

                if ui.menu_item(im_str!("Reset")).enabled(emu_running).build() {
                    if let Some(ref mut emu) = self.emu {
                        emu.reset().expect("error during reset");
                    }
                }

                self.gui.should_quit = ui.menu_item(im_str!("Exit")).build();
            });

            // Show debug-related menus in debug mode only
            if self.gui.debug {
                ui.menu(im_str!("Hardware")).build(|| {
                    if ui
                        .menu_item(im_str!("Memory Map"))
                        .enabled(emu_running)
                        .build()
                    {
                        self.gui
                            .views
                            .entry(View::MemMap)
                            .or_insert_with(|| box MemMapView::new());
                    }

                    if ui
                        .menu_item(im_str!("Peripherals"))
                        .enabled(emu_running)
                        .build()
                    {
                        self.gui
                            .views
                            .entry(View::Peripherals)
                            .or_insert_with(|| box PeripheralView::new());
                    }
                });

                ui.menu(im_str!("Debugging")).build(|| {
                    if ui
                        .menu_item(im_str!("Debugger"))
                        .enabled(emu_running)
                        .build()
                    {
                        self.gui
                            .views
                            .entry(View::Debugger)
                            .or_insert_with(|| box DebuggerView::new());
                    }

                    if ui
                        .menu_item(im_str!("Disassembler"))
                        .enabled(emu_running)
                        .build()
                    {
                        self.gui
                            .views
                            .entry(View::Disassembly)
                            .or_insert_with(|| box DisassemblyView::new());
                    }

                    if ui
                        .menu_item(im_str!("Memory Editor"))
                        .enabled(emu_running)
                        .build()
                    {
                        self.gui
                            .views
                            .entry(View::MemEditor)
                            .or_insert_with(|| box MemEditView::new());
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
        use imgui::{ImStr, ImString};

        ui.window(ImStr::new(&ImString::from(format!(
            "Screen - {:.0} FPS",
            ui.framerate()
        ))))
        .size(
            (EMU_X_RES as f32 + 15.0, EMU_Y_RES as f32 + 40.0),
            ImGuiCond::FirstUseEver,
        )
        .position((745.0, 30.0), ImGuiCond::FirstUseEver)
        .resizable(false)
        .build(|| {
            ui.image(self.vpu_texture, (EMU_X_RES as f32, EMU_Y_RES as f32))
                .build();
        });
    }
}
