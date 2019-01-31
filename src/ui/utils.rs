use imgui::{ImStr, ImString, Ui};

use std::ops::Range;
use std::path::PathBuf;
use std::time::Duration;

pub const DARK_GREY: [f32; 4] = [0.6, 0.6, 0.6, 1.0];
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
        use std::str::pattern::Pattern;
        "/".is_suffix_of(s.to_str())
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

        ui.open_popup(ImStr::new(&self.title));

        ui.popup_modal(ImStr::new(&self.title))
            .resizable(false)
            .always_auto_resize(true)
            .build(|| {
                let fl = self
                    .file_list
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>();

                clicked = ui.list_box(im_str!(""), &mut selected, &fl, 10);

                if ui.button(im_str!("Cancel"), (0.0, 0.0)) {
                    ui.close_current_popup();
                    on_result(None);
                }
            });

        // Update internal state
        self.click_timer = self.click_timer.map_or_else(
            || None,
            |v| v.checked_sub(Duration::from_float_secs(f64::from(delta_s))),
        );

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
pub fn list_clipper<F>(ui: &Ui, count: usize, f: F)
where
    F: Fn(Range<usize>),
{
    use imgui_sys::{
        ImGuiListClipper, ImGuiListClipper_Begin, ImGuiListClipper_End,
        ImGuiListClipper_GetDisplayEnd, ImGuiListClipper_GetDisplayStart, ImGuiListClipper_Step,
    };

    let font_height = ui.get_text_line_height_with_spacing();

    let mut clipper = ImGuiListClipper {
        start_pos_y: 0.0,
        items_height: -1.0,
        items_count: -1,
        step_no: 0,
        display_start: 0,
        display_end: 0,
    };

    unsafe {
        ImGuiListClipper_Begin(
            &mut clipper as *mut ImGuiListClipper,
            count as std::os::raw::c_int,
            font_height as std::os::raw::c_float,
        );
    }

    while unsafe { ImGuiListClipper_Step(&mut clipper as *mut ImGuiListClipper) } {
        let (start, end) = unsafe {
            (
                ImGuiListClipper_GetDisplayStart(&mut clipper as *mut ImGuiListClipper) as usize,
                ImGuiListClipper_GetDisplayEnd(&mut clipper as *mut ImGuiListClipper) as usize,
            )
        };
        f(start..end);
    }

    unsafe {
        ImGuiListClipper_End(&mut clipper as *mut ImGuiListClipper);
    }
}
