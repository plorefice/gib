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

    stepping: bool,
    step_into: bool,
    trace_event: Option<dbg::TraceEvent>,
    break_on_exception: bool,
}

impl EmuState {
    fn with(gb: GameBoy) -> EmuState {
        EmuState {
            gb,
            reset: false,

            stepping: false,
            step_into: false,
            trace_event: None,
            break_on_exception: true,
        }
    }
}

#[derive(Default)]
pub struct GuiState {
    debug: bool,
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
    pub fn new(debug: bool) -> EmuUi {
        let mut gui = GuiState::default();
        gui.debug = debug;

        EmuUi {
            ctx: Rc::from(RefCell::new(UiContext::new())),

            emu: None,
            gui,
            vpu_texture: None,

            disasm: None,
            debugger: None,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), dbg::TraceEvent> {
        self.emu = Some(EmuState::with(GameBoy::with_cartridge(rom)?));

        if self.gui.debug {
            let emu = match self.emu {
                Some(ref emu) => emu,
                None => panic!("initialized above"),
            };

            self.disasm = Some(DisasmWindow::new(emu));
            self.debugger = Some(DebuggerWindow::new());
        }
        Ok(())
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
                let res = if emu.stepping && emu.step_into {
                    emu.gb.step()
                } else if !emu.stepping {
                    emu.gb.run_for_vblank()
                } else {
                    Ok(())
                };

                if let Err(evt) = res {
                    emu.trace_event = Some(evt);

                    if emu.break_on_exception {
                        emu.stepping = true;
                    }
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

        if self.emu.is_some() {
            self.draw_screen(ui);
        }

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

            if let Err(evt) = self.load_rom(&rom[..]) {
                ui.popup_modal(im_str!("Error loading ROM")).build(|| {
                    ui.text(format!("{}", evt));
                });
                ui.open_popup(im_str!("Error loading ROM"));
            }
        }
    }

    fn draw_screen(&mut self, ui: &Ui) {
        ui.window(im_str!("Screen"))
            .size(
                (EMU_X_RES as f32 + 15.0, EMU_Y_RES as f32 + 40.0),
                ImGuiCond::FirstUseEver,
            )
            .position((430.0, 300.0), ImGuiCond::FirstUseEver)
            .resizable(false)
            .build(|| {
                if let Some(texture) = self.vpu_texture {
                    ui.image(texture, (EMU_X_RES as f32, EMU_Y_RES as f32))
                        .build();
                }
            });
    }
}
