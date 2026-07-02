use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::{gio, gdk};
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

/// Перестраивает панель закреплённых приложений из БД
pub fn rebuild(window: &MainWindow, db: &Rc<AppsDb>) {
    let imp = window.imp();

    while let Some(child) = imp.pinned_panel.first_child() {
        imp.pinned_panel.remove(&child);
    }

    let pinned_ids = db.pinned_ids().unwrap_or_default();

    for app in gio::AppInfo::all() {
        let id = match app.id() {
            Some(id) => id.to_string(),
            None => continue,
        };

        if !pinned_ids.contains(&id) {
            continue;
        }

        imp.pinned_panel.append(&build_pinned_icon(&app, &id, db, window));
    }
}

fn build_pinned_icon(app: &gio::AppInfo, app_id: &str, db: &Rc<AppsDb>, window: &MainWindow) -> gtk4::Widget {
    let image = gtk4::Image::new();
    image.set_pixel_size(32);
    if let Some(icon) = app.icon() {
        image.set_from_gicon(&icon);
    } else {
        image.set_icon_name(Some("application-x-executable"));
    }

    let button = gtk4::Button::new();
    button.set_child(Some(&image));
    button.add_css_class("flat");
    button.add_css_class("pinned-icon");
    button.set_tooltip_text(Some(&app.display_name()));

    // левый клик — запуск
    let app_clone = app.clone();
    let window_clone = window.clone();
    button.connect_clicked(move |_| {
        if let Err(e) = app_clone.launch(&[], gio::AppLaunchContext::NONE) {
            eprintln!("Не удалось запустить приложение: {}", e);
        } else {
            window_clone.close();
        }
    });

    // ПКМ — открывает меню Open / Unpin
    attach_context_menu(&button, app, app_id, db, window);

    button.upcast()
}

fn attach_context_menu(button: &gtk4::Button, app: &gio::AppInfo, app_id: &str, db: &Rc<AppsDb>, window: &MainWindow) {
    let menu_model = gio::Menu::new();
    menu_model.append(Some("Open"), Some("pinned.open"));
    menu_model.append(Some("Unpin"), Some("pinned.unpin"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(button);

    let popover_for_destroy = popover.clone();
    button.connect_destroy(move |_| {
        popover_for_destroy.unparent();
    });

    let action_group = gio::SimpleActionGroup::new();

    let app_open = app.clone();
    let action_open = gio::SimpleAction::new("open", None);
    action_open.connect_activate(move |_, _| {
        if let Err(e) = app_open.launch(&[], gio::AppLaunchContext::NONE) {
            eprintln!("Не удалось запустить приложение: {}", e);
        }
    });

    let id_unpin = app_id.to_string();
    let name_unpin = app.display_name().to_string();
    let db_unpin = db.clone();
    let window_unpin = window.clone();
    let action_unpin = gio::SimpleAction::new("unpin", None);
    action_unpin.connect_activate(move |_, _| {
        if let Err(e) = db_unpin.set_pinned(&id_unpin, false) {
            eprintln!("Ошибка открепления: {}", e);
        } else {
            println!("[Панель пинов] Откреплён: {}", name_unpin);
            window_unpin.rebuild_apps(&db_unpin);
            window_unpin.rebuild_pinned(&db_unpin);
        }
    });

    action_group.add_action(&action_open);
    action_group.add_action(&action_unpin);
    button.insert_action_group("pinned", Some(&action_group));

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(gdk::BUTTON_SECONDARY);

    let popover_clone = popover.clone();
    gesture.connect_pressed(move |gesture, _n_press, x, y| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        popover_clone.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover_clone.popup();
    });

    button.add_controller(gesture);
}
