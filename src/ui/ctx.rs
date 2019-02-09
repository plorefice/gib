use glium::glutin;
use glium::glutin::{
    ElementState::Pressed, Event, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode as Key,
    WindowEvent::*,
};
use glium::{Display, Surface};
use imgui::{FontGlyphRange, FrameSize, ImFontConfig, ImGui, Ui};
use imgui_glium_renderer::Renderer;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct UiContext {
    pub imgui: ImGui,
    pub display: Display,
    pub renderer: Renderer,
    pub events_loop: Rc<RefCell<glutin::EventsLoop>>,
    pub hidpi_factor: f64,

    mouse_state: MouseState,
    key_state: HashSet<Key>,
    should_quit: bool,
    focused: bool,
}

impl UiContext {
    /// Creates a new UI context with a window size of (width, height).
    pub fn new(width: f64, height: f64) -> UiContext {
        let events_loop = glutin::EventsLoop::new();

        let context = glutin::ContextBuilder::new().with_vsync(true);
        let builder = glutin::WindowBuilder::new()
            .with_title("gib")
            .with_dimensions(glutin::dpi::LogicalSize::new(width, height));

        let display = Display::new(builder, context, &events_loop).unwrap();

        let mut imgui = ImGui::init();
        imgui.set_ini_filename(None);

        let hidpi_factor = display.gl_window().get_hidpi_factor().round();

        UiContext::load_fonts(&mut imgui, hidpi_factor);
        UiContext::configure_keys(&mut imgui);

        let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        UiContext {
            imgui,
            display,
            renderer,
            events_loop: Rc::new(RefCell::from(events_loop)),
            hidpi_factor,

            mouse_state: MouseState::default(),
            key_state: HashSet::new(),
            should_quit: false,
            focused: true,
        }
    }

    pub fn poll_events(&mut self) {
        let events_loop = self.events_loop.clone();
        let win_hidpi_factor = self.window().get_hidpi_factor();

        events_loop.borrow_mut().poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    Focused(focus) => self.focused = focus,
                    CloseRequested => {
                        self.should_quit = true;
                    }
                    CursorMoved { position: pos, .. } => {
                        self.mouse_state.pos = pos
                            .to_physical(win_hidpi_factor)
                            .to_logical(self.hidpi_factor)
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
                            .to_physical(win_hidpi_factor)
                            .to_logical(self.hidpi_factor)
                            .y as f32;
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

                        self.update_imgui_keys(input, pressed);
                    }
                    ReceivedCharacter(c) => self.imgui.add_input_character(c),
                    _ => (),
                }
            }
        });

        self.update_imgui_mouse();
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn render<F>(&mut self, delta_s: f32, mut f: F)
    where
        F: FnMut(&Ui),
    {
        let window = self.display.gl_window();
        let win_hidpi_factor = window.get_hidpi_factor();

        let physical_size = window
            .get_inner_size()
            .unwrap()
            .to_physical(win_hidpi_factor);
        let logical_size = physical_size.to_logical(self.hidpi_factor);

        let frame_size = FrameSize {
            logical_size: logical_size.into(),
            hidpi_factor: self.hidpi_factor,
        };

        let ui = self.imgui.frame(frame_size, delta_s);
        f(&ui);

        let mut target = self.display.draw();
        target.clear_color(0.4, 0.5, 0.6, 1.0);
        self.renderer
            .render(&mut target, ui)
            .expect("Rendering failed");
        target.finish().unwrap();

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

    fn window(&self) -> std::cell::Ref<'_, glutin::GlWindow> {
        self.display.gl_window()
    }

    fn load_fonts(imgui: &mut ImGui, hidpi_factor: f64) {
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

    fn configure_keys(imgui: &mut ImGui) {
        use imgui::ImGuiKey;

        imgui.set_imgui_key(ImGuiKey::Tab, 0);
        imgui.set_imgui_key(ImGuiKey::LeftArrow, 1);
        imgui.set_imgui_key(ImGuiKey::RightArrow, 2);
        imgui.set_imgui_key(ImGuiKey::UpArrow, 3);
        imgui.set_imgui_key(ImGuiKey::DownArrow, 4);
        imgui.set_imgui_key(ImGuiKey::PageUp, 5);
        imgui.set_imgui_key(ImGuiKey::PageDown, 6);
        imgui.set_imgui_key(ImGuiKey::Home, 7);
        imgui.set_imgui_key(ImGuiKey::End, 8);
        imgui.set_imgui_key(ImGuiKey::Delete, 9);
        imgui.set_imgui_key(ImGuiKey::Backspace, 10);
        imgui.set_imgui_key(ImGuiKey::Enter, 11);
        imgui.set_imgui_key(ImGuiKey::Escape, 12);
        imgui.set_imgui_key(ImGuiKey::A, 13);
        imgui.set_imgui_key(ImGuiKey::C, 14);
        imgui.set_imgui_key(ImGuiKey::V, 15);
        imgui.set_imgui_key(ImGuiKey::X, 16);
        imgui.set_imgui_key(ImGuiKey::Y, 17);
        imgui.set_imgui_key(ImGuiKey::Z, 18);
    }

    fn update_imgui_mouse(&mut self) {
        self.imgui
            .set_mouse_pos(self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32);
        self.imgui.set_mouse_down([
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ]);
        self.imgui.set_mouse_wheel(self.mouse_state.wheel);
        self.mouse_state.wheel = 0.0;
    }

    /// Updates ImGUI's internal state with the key's new pressed state.
    fn update_imgui_keys(&mut self, input: glutin::KeyboardInput, pressed: bool) {
        use glium::glutin::VirtualKeyCode as Key;

        match input.virtual_keycode {
            Some(Key::Tab) => self.imgui.set_key(0, pressed),
            Some(Key::Left) => self.imgui.set_key(1, pressed),
            Some(Key::Right) => self.imgui.set_key(2, pressed),
            Some(Key::Up) => self.imgui.set_key(3, pressed),
            Some(Key::Down) => self.imgui.set_key(4, pressed),
            Some(Key::PageUp) => self.imgui.set_key(5, pressed),
            Some(Key::PageDown) => self.imgui.set_key(6, pressed),
            Some(Key::Home) => self.imgui.set_key(7, pressed),
            Some(Key::End) => self.imgui.set_key(8, pressed),
            Some(Key::Delete) => self.imgui.set_key(9, pressed),
            Some(Key::Back) => self.imgui.set_key(10, pressed),
            Some(Key::Return) => self.imgui.set_key(11, pressed),
            Some(Key::Escape) => self.imgui.set_key(12, pressed),
            Some(Key::A) => self.imgui.set_key(13, pressed),
            Some(Key::C) => self.imgui.set_key(14, pressed),
            Some(Key::V) => self.imgui.set_key(15, pressed),
            Some(Key::X) => self.imgui.set_key(16, pressed),
            Some(Key::Y) => self.imgui.set_key(17, pressed),
            Some(Key::Z) => self.imgui.set_key(18, pressed),
            Some(Key::LControl) | Some(Key::RControl) => self.imgui.set_key_ctrl(pressed),
            Some(Key::LShift) | Some(Key::RShift) => self.imgui.set_key_shift(pressed),
            Some(Key::LAlt) | Some(Key::RAlt) => self.imgui.set_key_alt(pressed),
            Some(Key::LWin) | Some(Key::RWin) => self.imgui.set_key_super(pressed),
            _ => {}
        }
    }
}
