pub fn address_edit_ui(ui: &mut egui::Ui, name: &str, buf: &mut String, editable: bool) -> bool {
    ui.horizontal(|ui| {
        ui.label(name);

        let response = egui::TextEdit::singleline(buf)
            .interactive(editable)
            .desired_width(37.)
            .show(ui)
            .response;

        // Respond to enter keypress
        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
    })
    .inner
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

/// Formats a 16-bit unsigned number into an hex string.
pub fn hexify(n: impl Into<u16>) -> String {
    format!("{:04X}", n.into())
}
