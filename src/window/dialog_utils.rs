use gtk4::prelude::*;

/// Добавляет обработчик Escape, закрывающий переданное окно
pub fn close_on_escape(window: &gtk4::Window) {
    let controller = gtk4::EventControllerKey::new();
    let window_clone = window.clone();

    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            window_clone.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });

    window.add_controller(controller);
}

use gtk4::glib;