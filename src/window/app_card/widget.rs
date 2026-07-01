use gtk4::prelude::*;
use gtk4::gio;

/// Строит только визуальную часть карточки: иконка + подпись, без меню и кликов
pub fn build_content(app: &gio::AppInfo) -> gtk4::Box {
    let name = app.display_name();

    let image = gtk4::Image::new();
    image.set_pixel_size(48);
    if let Some(icon) = app.icon() {
        image.set_from_gicon(&icon);
    } else {
        image.set_icon_name(Some("application-x-executable"));
    }

    let label = gtk4::Label::new(Some(&name));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_max_width_chars(12);
    label.set_lines(2);
    label.set_wrap(true);
    label.set_justify(gtk4::Justification::Center);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    content.append(&image);
    content.append(&label);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content
}