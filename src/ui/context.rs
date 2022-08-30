use std::{cell::RefCell, collections::HashSet, rc::Rc, time::Duration};

use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, TextureId, Ui};
use imgui_wgpu::{Renderer, RendererConfig, Texture};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pollster::block_on;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

#[cfg(target_os = "windows")]
use winit::platform::windows::WindowBuilderExtWindows;

use super::{EMU_X_RES, EMU_Y_RES};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct UiContext {
    imgui: Context,
    platform: WinitPlatform,
    event_loop: Rc<RefCell<EventLoop<()>>>,

    // Render system components
    window: Window,
    renderer: Renderer,
    device: wgpu::Device,
    surface: wgpu::Surface,
    queue: wgpu::Queue,

    key_state: HashSet<VirtualKeyCode>,
    should_quit: bool,
    focused: bool,
}

impl UiContext {
    /// Creates a new UI context with a window size of (width, height).
    pub fn new(width: f64, height: f64) -> UiContext {
        let event_loop = EventLoop::new();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        // Create native window surface
        let (window, size, surface) = {
            let builder = WindowBuilder::new()
                .with_title("gib")
                .with_inner_size(LogicalSize::new(width, height));

            #[cfg(target_os = "windows")]
            // NOTE(windows): enabling drag-and-drop causes cpal to panic.
            let builder = builder.with_drag_and_drop(false);

            let window = builder.build(&event_loop).expect("Window builder error");

            let size = window.inner_size();

            let surface = unsafe { instance.create_surface(&window) };

            (window, size, surface)
        };

        let hidpi_factor = window.scale_factor();

        // Retrieve graphics adapter
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No adapater available");

        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        // Set up surface
        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_desc);

        // Set up imgui
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

        UiContext::load_fonts(&mut imgui, hidpi_factor);

        // Set up renderer
        let renderer_config = RendererConfig {
            texture_format: surface_desc.format,
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
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Focused(focus) => self.focused = *focus,
                        WindowEvent::Resized(_) => {
                            let size = self.window.inner_size();

                            let surface_desc = wgpu::SurfaceConfiguration {
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                width: size.width as u32,
                                height: size.height as u32,
                                present_mode: wgpu::PresentMode::Mailbox,
                            };

                            self.surface.configure(&self.device, &surface_desc);
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
                    Event::RedrawRequested(id) if id == self.window.id() => do_render = true,
                    _ => (),
                }

                *control_flow = ControlFlow::Exit;
            });

        do_render
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Creates a new texture displaying the currently emulated screen,
    /// ready to be presented during the next rendering step.
    pub fn prepare_screen_texture(
        &mut self,
        texture_id: &mut Option<TextureId>,
        vpu_buffer: &[u8],
    ) {
        let size = wgpu::Extent3d {
            width: EMU_X_RES as u32,
            height: EMU_Y_RES as u32,
            ..Default::default()
        };

        // Create the wgpu texture
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        // Extract the texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the texture sampler
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        // HACK: this is taken from the imgui_gpu::Renderer internals, so it may break sooner or later.
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create the texture bind group from the layout
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Write data into the texture
        let texture = Texture::from_raw_parts(texture, view, bind_group, size);
        texture.write(&self.queue, vpu_buffer, EMU_X_RES as u32, EMU_Y_RES as u32);

        // If this is the first time rendering, insert the new texture, otherwise replace an existing one
        if let Some(ref mut vpu_texture) = texture_id {
            self.renderer.textures.replace(*vpu_texture, texture);
        } else {
            *texture_id = Some(self.renderer.textures.insert(texture));
        }
    }

    // Perform the rendering pass of the ui.
    pub fn render<F>(&mut self, delta: Duration, mut f: F)
    where
        F: FnMut(&Ui),
    {
        let io = self.imgui.io_mut();

        io.update_delta_time(delta);

        let frame = match self.surface.get_current_texture() {
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

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
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

        frame.present();

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
                data: include_bytes!("../../assets/mplus-1p-regular.ttf"),
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
