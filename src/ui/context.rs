use std::{cell::RefCell, collections::HashSet, rc::Rc, time::Duration};

use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pollster::block_on;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::{run_return::EventLoopExtRunReturn, windows::WindowBuilderExtWindows},
    window::{Window, WindowBuilder},
};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct UiContext {
    pub imgui: Context,
    pub platform: WinitPlatform,

    pub window: Window,
    pub renderer: Renderer,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub swap_chain: wgpu::SwapChain,
    pub queue: wgpu::Queue,

    pub event_loop: Rc<RefCell<EventLoop<()>>>,

    key_state: HashSet<VirtualKeyCode>,
    should_quit: bool,
    focused: bool,
}

impl UiContext {
    /// Creates a new UI context with a window size of (width, height).
    pub fn new(width: f64, height: f64) -> UiContext {
        let event_loop = EventLoop::new();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        // Create native window surface
        let (window, size, surface) = {
            let window = WindowBuilder::new()
                .with_title("gib")
                .with_drag_and_drop(false) // NOTE(windows): see function doc
                .with_inner_size(LogicalSize::new(width, height))
                .build(&event_loop)
                .expect("Window builder error");

            let size = window.inner_size();

            let surface = unsafe { instance.create_surface(&window) };

            (window, size, surface)
        };

        let hidpi_factor = window.scale_factor();

        // Retrieve graphics adapter
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        }))
        .expect("No adapater available");

        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        // Set up swap chain
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // Set up imgui
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

        UiContext::load_fonts(&mut imgui, hidpi_factor);

        // Set up renderer
        let renderer_config = RendererConfig {
            texture_format: sc_desc.format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

        UiContext {
            imgui,
            platform,

            window,
            renderer,
            device,
            surface,
            swap_chain,
            queue,

            event_loop: Rc::new(RefCell::from(event_loop)),

            key_state: HashSet::new(),
            should_quit: false,
            focused: true,
        }
    }

    pub fn poll_events(&mut self) -> bool {
        let mut do_render = false;

        let event_loop = self.event_loop.clone();

        event_loop
            .borrow_mut()
            .run_return(|event, _, control_flow| {
                self.platform
                    .handle_event(self.imgui.io_mut(), &self.window, &event);

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Focused(focus) => self.focused = focus,
                        WindowEvent::Resized(_) => {
                            let size = self.window.inner_size();

                            let sc_desc = wgpu::SwapChainDescriptor {
                                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                width: size.width as u32,
                                height: size.height as u32,
                                present_mode: wgpu::PresentMode::Mailbox,
                            };

                            self.swap_chain =
                                self.device.create_swap_chain(&self.surface, &sc_desc);
                        }
                        WindowEvent::CloseRequested => {
                            self.should_quit = true;
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            let pressed = input.state == ElementState::Pressed;

                            if let Some(vk) = input.virtual_keycode {
                                if pressed {
                                    self.key_state.insert(vk);
                                } else {
                                    self.key_state.remove(&vk);
                                }
                            }
                        }
                        _ => (),
                    },
                    Event::MainEventsCleared => self.window.request_redraw(),
                    Event::RedrawEventsCleared => do_render = true,
                    _ => (),
                }

                *control_flow = ControlFlow::Exit;
            });

        do_render
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn render<F>(&mut self, delta: Duration, mut f: F)
    where
        F: FnMut(&Ui),
    {
        let io = self.imgui.io_mut();

        io.update_delta_time(delta);

        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Dropped frame: {}", e);
                return;
            }
        };

        self.platform
            .prepare_frame(io, &self.window)
            .expect("Preparing frame");

        let ui = self.imgui.frame();
        f(&ui);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        self.platform.prepare_render(&ui, &self.window);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.4,
                        g: 0.5,
                        b: 0.6,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.renderer
            .render(ui.render(), &self.queue, &self.device, &mut rpass)
            .expect("Rendering failed");

        drop(rpass);

        self.queue.submit(Some(encoder.finish()));

        if !self.focused {
            // Throttle to 60 fps when in background, since macOS doesn't honor
            // V-Sync settings for non-visible windows, making the CPU shoot to 100%.
            std::thread::sleep(std::time::Duration::from_nanos(1_000_000_000 / 60));
        }
    }

    /// Returns the pressed state for the given virtual key.
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
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

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    }
}
