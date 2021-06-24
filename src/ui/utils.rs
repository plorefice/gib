use imgui::{im_str, ImStr, ImString, Ui};

use std::ops::Range;
use std::path::PathBuf;
use std::time::Duration;

pub const DARK_GREY: [f32; 4] = [0.6, 0.6, 0.6, 1.0];
pub const DARK_GREEN: [f32; 4] = [0.0, 0.2, 0.0, 1.0];
pub const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

pub struct FileDialog {
    title: ImString,
    current_dir: PathBuf,
    file_list: Vec<ImString>,
    click_timer: Option<Duration>,
}

impl FileDialog {
    pub fn new<T>(title: T) -> FileDialog
    where
        T: Into<String>,
    {
        use std::env::current_dir;

        let mut fd = FileDialog {
            title: ImString::new(title),
            current_dir: current_dir().unwrap(),
            file_list: vec![],
            click_timer: None,
        };

        fd.chdir();
        fd
    }

    fn is_dir(s: &ImStr) -> bool {
        s.to_str().ends_with('/')
    }

    fn chdir(&mut self) {
        use std::cmp::Ordering;

        self.file_list = std::fs::read_dir(&self.current_dir)
            .unwrap()
            .map(|de| {
                let de = de.unwrap();
                let mut n = de.file_name().into_string().unwrap();

                if de.file_type().unwrap().is_dir() {
                    n += "/";
                }
                ImString::from(n)
            })
            .collect::<Vec<_>>();

        self.file_list.sort_by(|a, b| {
            let a_is_dir = FileDialog::is_dir(a);
            let b_is_dir = FileDialog::is_dir(b);

            if (a_is_dir && b_is_dir) || (!a_is_dir && !b_is_dir) {
                a.cmp(b)
            } else if a_is_dir {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        // Prepend the parent directory to the listing
        self.file_list
            .splice(0..0, [ImString::from(String::from("../"))].iter().cloned());
    }

    pub fn build<F>(&mut self, delta_s: f32, ui: &Ui, mut on_result: F)
    where
        F: FnMut(Option<PathBuf>),
    {
        let mut selected = 0;
        let mut clicked = false;

        ui.open_popup(&self.title);

        ui.popup_modal(&self.title)
            .resizable(false)
            .always_auto_resize(true)
            .build(|| {
                let fl = self
                    .file_list
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<&ImStr>>();

                clicked = ui.list_box(im_str!(""), &mut selected, &fl, 10);

                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                    ui.close_current_popup();
                    on_result(None);
                }
            });

        // Update internal state
        self.click_timer = self
            .click_timer
            .map_or_else(|| None, |v| v.checked_sub(Duration::from_secs_f32(delta_s)));

        // Check for double clicks
        if clicked {
            if self.click_timer.is_some() {
                let selection = &self.file_list[selected as usize];

                if FileDialog::is_dir(selection) {
                    self.current_dir.push(selection.to_str());
                    self.chdir();
                } else {
                    on_result(Some(
                        PathBuf::from(&self.current_dir).join(selection.to_str()),
                    ));
                }
            } else {
                self.click_timer = Some(Duration::from_millis(200));
            }
        }
    }
}

/// Safe wrapper around [`ImGuiListClipper`](imgui_sys.ImGuiListClipper).
pub fn list_clipper<F>(ui: &Ui, count: usize, mut f: F)
where
    F: FnMut(Range<usize>),
{
    use imgui_sys::{
        ImGuiListClipper, ImGuiListClipper_Begin, ImGuiListClipper_End, ImGuiListClipper_Step,
    };

    let font_height = ui.text_line_height_with_spacing();

    let mut clipper = ImGuiListClipper {
        StartPosY: 0.0,
        ItemsHeight: -1.0,
        ItemsCount: -1,
        StepNo: 0,
        DisplayStart: 0,
        DisplayEnd: 0,
    };

    unsafe {
        ImGuiListClipper_Begin(
            &mut clipper as *mut ImGuiListClipper,
            count as std::os::raw::c_int,
            font_height as std::os::raw::c_float,
        );
    }

    while unsafe { ImGuiListClipper_Step(&mut clipper as *mut ImGuiListClipper) } {
        f(clipper.DisplayStart as usize..clipper.DisplayEnd as usize);
    }

    unsafe {
        ImGuiListClipper_End(&mut clipper as *mut ImGuiListClipper);
    }
}

pub fn input_addr(ui: &Ui, name: &str, val: &mut Option<u16>, editable: bool) {
    let mut buf = if let Some(v) = val {
        ImString::from(format!("{:04X}", v))
    } else {
        ImString::with_capacity(4)
    };

    let tok = ui.push_item_width(37.0);

    ui.input_text(im_str!("{}", name).as_ref(), &mut buf)
        .chars_hexadecimal(true)
        .chars_noblank(true)
        .chars_uppercase(true)
        .auto_select_all(true)
        .read_only(!editable)
        .build();

    drop(tok);

    *val = u16::from_str_radix(buf.to_str(), 16).ok();
}

/// Scrolls the view to the specified `line`.
/// If `content_height` is also specified, it will scroll in such a way that the line
/// is exactly in the middle of the view.
///
/// This function assumes that all lines are textual and of fixed height.
pub fn scroll_to(ui: &Ui, line: usize, content_height: Option<f32>) {
    unsafe {
        imgui_sys::igSetScrollY(
            ui.text_line_height_with_spacing() * line as f32
                - content_height.unwrap_or_default() / 2.0,
        );
    }
}

/// Converts a slice of bytes into its ASCII representation
/// if the corresponding character is visible, otherwise into a '.'.
pub fn format_ascii(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() + 2);
    s.push('|');
    for &d in data {
        s.push(if !d.is_ascii() || d.is_ascii_control() {
            '.'
        } else {
            unsafe { std::char::from_u32_unchecked(d.into()) }
        });
    }
    s.push('|');
    s
}
