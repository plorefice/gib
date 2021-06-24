use glutin::dpi::LogicalSize;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_gfx_renderer::{Renderer, Shaders};

use gfx_core::handle::{DepthStencilView, RenderTargetView};
use gfx_device_gl::{Device, Factory, Resources};
use glutin::{ContextBuilder, EventsLoop, VirtualKeyCode as Key, WindowBuilder, WindowedContext};
use imgui_winit_support::WinitPlatform;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct UiContext {
    pub imgui: Context,
    pub platform: WinitPlatform,

    pub renderer: Renderer<ColorFormat, Resources>,
    pub windowed_context: WindowedContext,
    pub device: Device,
    pub factory: Factory,
    pub main_color: RenderTargetView<Resources, ColorFormat>,
    pub main_depth: DepthStencilView<Resources, DepthFormat>,

    pub events_loop: Rc<RefCell<EventsLoop>>,
    pub hidpi_factor: f64,

    key_state: HashSet<Key>,
    should_quit: bool,
    focused: bool,
}

impl UiContext {
    /// Creates a new UI context with a window size of (width, height).
    pub fn new(width: f64, height: f64) -> UiContext {
        let events_loop = EventsLoop::new();

        let context = ContextBuilder::new().with_vsync(true);

        let builder = WindowBuilder::new()
            .with_title("gib")
            .with_dimensions(LogicalSize::new(width, height));

        let (windowed_context, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init(builder, context, &events_loop)
                .expect("Failed to initalize graphics");

        let shaders = {
            let version = device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                if version.minor >= 2 {
                    Shaders::GlSl150
                } else {
                    Shaders::GlSl130
                }
            } else {
                Shaders::GlSl110
            }
        };

        let mut imgui = Context::init();
        {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
                let x = col[0].powf(2.2);
                let y = col[1].powf(2.2);
                let z = col[2].powf(2.2);
                let w = 1.0 - (1.0 - col[3]).powf(2.2);
                [x, y, z, w]
            }

            let style = imgui.style_mut();
            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
        }
        imgui.set_ini_filename(None);

        //let hidpi_factor = window.get_hidpi_factor().round();
        let hidpi_factor = 1.;
        UiContext::load_fonts(&mut imgui, hidpi_factor);

        let mut platform = WinitPlatform::init(&mut imgui);

        let renderer = Renderer::init(&mut imgui, &mut factory, shaders)
            .expect("Failed to initialize renderer");

        UiContext {
            imgui,
            platform,

            renderer,
            windowed_context,
            device,
            factory,
            main_color,
            main_depth,

            events_loop: Rc::new(RefCell::from(events_loop)),
            hidpi_factor,

            key_state: HashSet::new(),
            should_quit: false,
            focused: true,
        }
    }

    pub fn poll_events(&mut self) {
        let events_loop = self.events_loop.clone();

        events_loop.borrow_mut().poll_events(|event| {
            use glutin::{
                ElementState::Pressed,
                Event,
                WindowEvent::{CloseRequested, Focused, KeyboardInput, Resized},
            };

            self.platform
                .handle_event(self.imgui.io_mut(), self.windowed_context.window(), &event);

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    Focused(focus) => self.focused = focus,
                    Resized(size) => {
                        gfx_window_glutin::update_views(
                            &self.windowed_context,
                            &mut self.main_color,
                            &mut self.main_depth,
                        );
                    }
                    CloseRequested => {
                        self.should_quit = true;
                    }
                    KeyboardInput { input, .. } => {
                        let pressed = input.state == Pressed;

                        if let Some(vk) = input.virtual_keycode {
                            if pressed {
                                self.key_state.insert(vk);
                            } else {
                                self.key_state.remove(&vk);
                            }
                        }
                    }
                    _ => (),
                }
            }
        });
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn render<F>(&mut self, delta_s: f32, mut f: F)
    where
        F: FnMut(&Ui),
    {
        use gfx::Device;

        self.platform
            .prepare_frame(self.imgui.io_mut(), self.windowed_context.window());

        let ui = self.imgui.frame();

        f(&ui);

        let mut encoder: gfx::Encoder<_, _> = self.factory.create_command_buffer().into();

        encoder.clear(&self.main_color, [0.4, 0.5, 0.6, 1.0]);

        self.platform
            .prepare_render(&ui, self.windowed_context.window());

        let draw_data = ui.render();

        self.renderer
            .render(
                &mut self.factory,
                &mut encoder,
                &mut self.main_color,
                draw_data,
            )
            .expect("Rendering failed");

        encoder.flush(&mut self.device);

        self.windowed_context.swap_buffers().unwrap();

        self.device.cleanup();

        if !self.focused {
            // Throttle to 60 fps when in background, since macOS doesn't honor
            // V-Sync settings for non-visible windows, making the CPU shoot to 100%.
            std::thread::sleep(std::time::Duration::from_nanos(1_000_000_000 / 60));
        }
    }

    /// Returns the pressed state for the given virtual key.
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.key_state.contains(&key)
    }

    fn load_fonts(imgui: &mut Context, hidpi_factor: f64) {
        let font_size = (13.0 * hidpi_factor) as f32;

        imgui.fonts().add_font(&[
            FontSource::DefaultFontData {
                config: Some(FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            },
            FontSource::TtfData {
                data: include_bytes!("../../res/mplus-1p-regular.ttf"),
                size_pixels: font_size,
                config: Some(FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    rasterizer_multiply: 1.75,
                    glyph_ranges: FontGlyphRanges::japanese(),
                    ..Default::default()
                }),
            },
        ]);

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);
    }
}
