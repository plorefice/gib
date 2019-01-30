use imgui::{ImColor, ImGuiCol, ImStr, ImString, ImVec2, ImVec4, Ui};

use std::path::PathBuf;
use std::time::Duration;

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

pub fn text_bg_color(ui: &Ui) -> ImVec4 {
    ui.imgui().style().colors[ImGuiCol::Button as usize]
}

pub fn text_with_bg<P, S, C>(ui: &Ui, pos: P, s: S, color: Option<C>)
where
    P: Into<ImVec2>,
    S: AsRef<ImStr>,
    C: Into<ImColor>,
{
    let ds = ui.calc_text_size(s.as_ref(), false, 0.0);
    let pos = pos.into();

    if let Some(c) = color {
        let (wx, wy) = ui.get_window_pos();

        ui.get_window_draw_list()
            .add_rect(
                [wx + pos.x - ds.x * 0.5, wy + pos.y - ds.y * 0.2],
                [wx + pos.x + ds.x * 1.5, wy + pos.y + ds.y * 1.2],
                c,
            )
            .filled(true)
            .build();
    }

    let (ox, oy) = ui.get_cursor_pos();
    ui.set_cursor_pos(pos);
    ui.text_wrapped(s.as_ref());
    let (nx, ny) = ui.get_cursor_pos();

    ui.set_cursor_pos((ox + (nx - pos.x), oy + (ny - pos.y)));
}
