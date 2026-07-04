mod context_menu;
mod edit_dialog;

use gtk4::prelude::*;
use gtk4::gio;
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

/// Строит одну карточку приложения: иконка + подпись + кнопка запуска + ПКМ-меню
pub fn build(app: &gio::AppInfo, db: &Rc<AppsDb>, window: &MainWindow) -> gtk4::FlowBoxChild {
    let name = app.display_name();
    let app_id = app.id().map(|s| s.to_string()).unwrap_or_default();

    let image = gtk4::Image::new();
    image.set_pixel_size(40);
    if let Some(icon) = app.icon() {
        image.set_from_gicon(&icon);
    } else {
        image.set_icon_name(Some("application-x-executable"));
    }

    let label = gtk4::Label::new(Some(&name));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_max_width_chars(7);
    label.set_lines(1);
    label.set_wrap(false);
    label.set_justify(gtk4::Justification::Center);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    content.set_margin_top(4);
    content.set_margin_bottom(4);
    content.set_margin_start(2);
    content.set_margin_end(2);
    content.append(&image);
    content.append(&label);

    let button = gtk4::Button::new();
    button.set_child(Some(&content));
    button.add_css_class("flat");
    button.add_css_class("app-tile");
    button.set_width_request(72);
    button.set_height_request(72);

    let app_clone = app.clone();
    let window_clone = window.clone();
    button.connect_clicked(move |_| {
        if let Err(e) = app_clone.launch(&[], gio::AppLaunchContext::NONE) {
            eprintln!("Не удалось запустить приложение: {}", e);
        } else {
            window_clone.close();
        }
    });

    let is_hidden_now = db.is_hidden(&app_id);
    let is_pinned_now = db.is_pinned(&app_id);
    let is_manual = app_id.starts_with("yggdrasil-manual-");
    
    context_menu::attach(&button, app, &app_id, &name, db, window, is_hidden_now, is_pinned_now, is_manual);

    let flow_child = gtk4::FlowBoxChild::new();
    flow_child.set_child(Some(&button));
    flow_child
}