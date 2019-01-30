use super::gb::*;

mod ctx;
mod debug;
mod disasm;
mod utils;

use ctx::UiContext;
use debug::DebuggerWindow;
use disasm::DisasmWindow;

use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use imgui::{ImGuiCond, Ui};

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

const EMU_X_RES: usize = 160;
const EMU_Y_RES: usize = 144;

pub struct EmuState {
    gb: GameBoy,
    reset: bool,
    running: bool,
    step_into: bool,
}

impl EmuState {
    fn with(gb: GameBoy) -> EmuState {
        EmuState {
            gb,
            reset: false,
            running: false,
            step_into: false,
        }
    }
}

#[derive(Default)]
pub struct GuiState {
    should_quit: bool,
    file_dialog: Option<utils::FileDialog>,
}

pub struct EmuUi {
    ctx: Rc<RefCell<UiContext>>,
    emu: Option<EmuState>,
    gui: GuiState,

    vpu_texture: Option<imgui::ImTexture>,

    disasm: Option<DisasmWindow>,
    debugger: Option<DebuggerWindow>,
}

impl EmuUi {
    pub fn new() -> EmuUi {
        EmuUi {
            ctx: Rc::from(RefCell::new(UiContext::new())),

            emu: None,
            gui: GuiState::default(),
            vpu_texture: None,

            disasm: None,
            debugger: None,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.emu = Some(EmuState::with(GameBoy::with_cartridge(rom)));
    }

    pub fn run(&mut self) {
        let mut last_frame = Instant::now();
        let mut vbuf = vec![0; EMU_X_RES * EMU_Y_RES * 4];

        loop {
            let ctx = self.ctx.clone();
            let mut ctx = ctx.borrow_mut();

            ctx.poll_events();

            if self.gui.should_quit || ctx.should_quit() {
                break;
            }

            let now = Instant::now();
            let delta = now - last_frame;
            let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
            last_frame = now;

            if let Some(ref mut emu) = self.emu {
                if emu.step_into {
                    emu.gb.single_step();
                } else if emu.running {
                    emu.gb.run_to_vblank();
                }

                emu.gb.rasterize(&mut vbuf[..]);

                let new_screen = Texture2d::new(
                    ctx.display.get_context(),
                    RawImage2d {
                        data: Cow::Borrowed(&vbuf[..]),
                        width: EMU_X_RES as u32,
                        height: EMU_Y_RES as u32,
                        format: ClientFormat::U8U8U8U8,
                    },
                )
                .unwrap();

                if let Some(texture) = self.vpu_texture {
                    ctx.renderer.textures().replace(texture, new_screen);
                } else {
                    self.vpu_texture = Some(ctx.renderer.textures().insert(new_screen));
                }
            }

            ctx.render(delta_s, |ui| self.draw(delta_s, ui));
        }
    }

    fn draw(&mut self, delta_s: f32, ui: &Ui) {
        self.draw_menu_bar(delta_s, ui);
        //self.draw_screen(ui);

        if let Some(ref mut emu) = self.emu {
            if let Some(ref mut view) = self.disasm {
                view.draw(ui, emu);
            }

            if let Some(ref mut view) = self.debugger {
                view.draw(ui, emu);
            }
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

                if ui.menu_item(im_str!("Reset")).enabled(emu_running).build() {
                    if let Some(ref mut emu) = self.emu {
                        emu.reset = true;
                    }
                }

                self.gui.should_quit = ui.menu_item(im_str!("Exit")).build();
            });

            ui.menu(im_str!("Hardware")).build(|| {
                ui.menu_item(im_str!("VPU")).enabled(emu_running).build();
                ui.menu_item(im_str!("APU")).enabled(emu_running).build();
                ui.menu_item(im_str!("TIM")).enabled(emu_running).build();
                ui.menu_item(im_str!("ITR")).enabled(emu_running).build();
            });

            ui.menu(im_str!("Debugging")).build(|| {
                if ui
                    .menu_item(im_str!("Debugger"))
                    .enabled(emu_running)
                    .build()
                    && self.debugger.is_none()
                {
                    self.debugger = Some(DebuggerWindow::new());
                }

                if ui
                    .menu_item(im_str!("Disassembler"))
                    .enabled(emu_running)
                    .build()
                    && self.disasm.is_none()
                {
                    if let Some(ref emu) = self.emu {
                        self.disasm = Some(DisasmWindow::new(emu));
                    }
                }

                ui.menu_item(im_str!("Memory Editor"))
                    .enabled(emu_running)
                    .build();
            })
        });
    }

    fn draw_file_dialog(&mut self, delta_s: f32, ui: &Ui) {
        use std::fs;

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

        if let Some(rom_file) = fd_chosen {
            let rom = fs::read(rom_file).unwrap();
            self.load_rom(&rom[..]);
        }
    }

    fn draw_screen(&mut self, ui: &Ui) {
        ui.window(im_str!("Screen"))
            .size(
                (EMU_X_RES as f32 + 15.0, EMU_Y_RES as f32 + 40.0),
                ImGuiCond::FirstUseEver,
            )
            .position((780.0, 10.0), ImGuiCond::FirstUseEver)
            .resizable(false)
            .build(|| {
                if let Some(texture) = self.vpu_texture {
                    ui.image(texture, (EMU_X_RES as f32, EMU_Y_RES as f32))
                        .build();
                }
            });
    }
}
