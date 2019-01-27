use super::gb::GameBoy;

use glium::glutin;
use glium::glutin::ElementState::Pressed;
use glium::glutin::WindowEvent::*;
use glium::glutin::{Event, MouseButton, MouseScrollDelta, TouchPhase};
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    Display, Surface, Texture2d,
};
use imgui::{FontGlyphRange, FrameSize, ImFontConfig, ImGui, ImGuiCond, Ui};
use imgui_glium_renderer::Renderer;

use std::borrow::Cow;
use std::time::Instant;

const EMU_X_RES: usize = 160;
const EMU_Y_RES: usize = 144;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct EmuUi {
    emu: GameBoy,
    mouse_state: MouseState,

    texture: imgui::ImTexture,
}

impl EmuUi {
    pub fn new(emu: GameBoy) -> EmuUi {
        EmuUi {
            emu,
            mouse_state: MouseState::default(),
            texture: imgui::ImTexture::from(0),
        }
    }

    pub fn run(&mut self) {
        let mut events_loop = glutin::EventsLoop::new();

        let context = glutin::ContextBuilder::new().with_vsync(true);
        let builder = glutin::WindowBuilder::new()
            .with_title("gb-rs")
            .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 764.0));

        let display = Display::new(builder, context, &events_loop).unwrap();
        let window = display.gl_window();

        let mut imgui = ImGui::init();
        imgui.set_ini_filename(None);

        let hidpi_factor = window.get_hidpi_factor().round();
        self.load_fonts(&mut imgui, hidpi_factor);

        let mut renderer =
            Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        let mut last_frame = Instant::now();
        let mut quit = false;

        loop {
            events_loop.poll_events(|event| {
                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        CloseRequested => quit = true,
                        CursorMoved { position: pos, .. } => {
                            self.mouse_state.pos = pos
                                .to_physical(window.get_hidpi_factor())
                                .to_logical(hidpi_factor)
                                .into();
                        }
                        MouseInput { state, button, .. } => match button {
                            MouseButton::Left => self.mouse_state.pressed.0 = state == Pressed,
                            MouseButton::Right => self.mouse_state.pressed.1 = state == Pressed,
                            MouseButton::Middle => self.mouse_state.pressed.2 = state == Pressed,
                            _ => {}
                        },
                        MouseWheel {
                            delta: MouseScrollDelta::LineDelta(_, y),
                            phase: TouchPhase::Moved,
                            ..
                        } => self.mouse_state.wheel = y,
                        MouseWheel {
                            delta: MouseScrollDelta::PixelDelta(pos),
                            phase: TouchPhase::Moved,
                            ..
                        } => {
                            self.mouse_state.wheel = pos
                                .to_physical(window.get_hidpi_factor())
                                .to_logical(hidpi_factor)
                                .y as f32;
                        }
                        ReceivedCharacter(c) => imgui.add_input_character(c),
                        _ => (),
                    }
                }
            });

            let now = Instant::now();
            let delta = now - last_frame;
            let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
            last_frame = now;

            self.update_mouse(&mut imgui);

            let mut vbuf = vec![0u8; EMU_X_RES * EMU_Y_RES * 3];

            self.emu.run_to_vblank();
            self.emu.rasterize(&mut vbuf[..]);

            let gl_texture = Texture2d::new(
                display.get_context(),
                RawImage2d {
                    data: Cow::Owned(vbuf),
                    width: 160,
                    height: 144,
                    format: ClientFormat::U8U8U8,
                },
            )
            .unwrap();

            renderer
                .textures()
                .replace(self.texture, gl_texture)
                .unwrap();

            let physical_size = window
                .get_inner_size()
                .unwrap()
                .to_physical(window.get_hidpi_factor());
            let logical_size = physical_size.to_logical(hidpi_factor);

            let frame_size = FrameSize {
                logical_size: logical_size.into(),
                hidpi_factor,
            };

            let ui = imgui.frame(frame_size, delta_s);
            if !self.render(&ui) {
                break;
            }

            let mut target = display.draw();
            target.clear_color(1.0, 1.0, 1.0, 1.0);
            renderer.render(&mut target, ui).expect("Rendering failed");
            target.finish().unwrap();

            if quit {
                break;
            }
        }
    }

    fn load_fonts(&mut self, imgui: &mut ImGui, hidpi_factor: f64) {
        let font_size = (13.0 * hidpi_factor) as f32;

        imgui.fonts().add_font_with_config(
            include_bytes!("../../res/mplus-1p-regular.ttf"),
            ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size)
                .rasterizer_multiply(1.75),
            &FontGlyphRange::japanese(),
        );

        imgui.fonts().add_default_font_with_config(
            ImFontConfig::new()
                .merge_mode(true)
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
        );

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);
    }

    fn update_mouse(&mut self, imgui: &mut ImGui) {
        imgui.set_mouse_pos(self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32);
        imgui.set_mouse_down([
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ]);
        imgui.set_mouse_wheel(self.mouse_state.wheel);
        self.mouse_state.wheel = 0.0;
    }

    fn render(&self, ui: &Ui) -> bool {
        ui.window(im_str!("Hello world"))
            .size((175.0, 180.0), ImGuiCond::FirstUseEver)
            .resizable(false)
            .build(|| {
                ui.image(self.texture, (EMU_X_RES as f32, EMU_Y_RES as f32))
                    .build();
            });

        true
    }
}
