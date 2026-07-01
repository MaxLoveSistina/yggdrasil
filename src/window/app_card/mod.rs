use gtk4::prelude::*;
use gtk4::{gio, gdk};
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
    button.connect_clicked(move |_| {
        if let Err(e) = app_clone.launch(&[], gio::AppLaunchContext::NONE) {
            eprintln!("Не удалось запустить приложение: {}", e);
        }
    });

    let is_hidden_now = db.is_hidden(&app_id);
    let is_pinned_now = db.is_pinned(&app_id);
    attach_context_menu(&button, app, &app_id, &name, db, window, is_hidden_now, is_pinned_now);

    let flow_child = gtk4::FlowBoxChild::new();
    flow_child.set_child(Some(&button));
    flow_child
}

fn attach_context_menu(
    button: &gtk4::Button,
    app: &gio::AppInfo,
    app_id: &str,
    app_name: &str,
    db: &Rc<AppsDb>,
    window: &MainWindow,
    is_hidden: bool,
    is_pinned: bool,
) {
    let hide_label = if is_hidden { "Unhide" } else { "Hide" };
    let pin_label = if is_pinned { "Unpin" } else { "Pin" };

    let menu_model = gio::Menu::new();
    menu_model.append(Some("Launch"), Some("app_card.launch"));
    menu_model.append(Some("Add to category"), Some("app_card.add_category"));
    menu_model.append(Some(pin_label), Some("app_card.pin"));
    menu_model.append(Some(hide_label), Some("app_card.hide"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(button);

    let popover_for_destroy = popover.clone();
    button.connect_destroy(move |_| {
        popover_for_destroy.unparent();
    });

    let action_group = build_actions(app, app_id, app_name, db, window, is_hidden, is_pinned);
    button.insert_action_group("app_card", Some(&action_group));

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

fn build_actions(
    app: &gio::AppInfo,
    app_id: &str,
    app_name: &str,
    db: &Rc<AppsDb>,
    window: &MainWindow,
    is_hidden: bool,
    is_pinned: bool,
) -> gio::SimpleActionGroup {
    let action_group = gio::SimpleActionGroup::new();

    let app_launch = app.clone();
    let name_launch = app_name.to_string();
    let action_launch = gio::SimpleAction::new("launch", None);
    action_launch.connect_activate(move |_, _| {
        println!("[Меню] Launch: {}", name_launch);
        if let Err(e) = app_launch.launch(&[], gio::AppLaunchContext::NONE) {
            eprintln!("Не удалось запустить приложение: {}", e);
        }
    });

    let id_category = app_id.to_string();
    let name_category = app_name.to_string();
    let db_category = db.clone();
    let window_category = window.clone();
    let action_add_category = gio::SimpleAction::new("add_category", None);
    action_add_category.connect_activate(move |_, _| {
        crate::window::add_to_category::show(&window_category, db_category.clone(), id_category.clone(), name_category.clone());
    });

    let id_pin = app_id.to_string();
    let name_pin = app_name.to_string();
    let db_pin = db.clone();
    let window_pin = window.clone();
    let new_pinned_state = !is_pinned;
    let action_pin = gio::SimpleAction::new("pin", None);
    action_pin.connect_activate(move |_, _| {
        if let Err(e) = db_pin.set_pinned(&id_pin, new_pinned_state) {
            eprintln!("Ошибка изменения закрепления: {}", e);
        } else {
            let verb = if new_pinned_state { "закреплён" } else { "откреплён" };
            println!("[Меню] {}: {}", name_pin, verb);
            window_pin.rebuild_apps(&db_pin);
            window_pin.rebuild_pinned(&db_pin);
        }
    });

    let id_hide = app_id.to_string();
    let name_hide = app_name.to_string();
    let db_hide = db.clone();
    let window_hide = window.clone();
    let new_hidden_state = !is_hidden;
    let action_hide = gio::SimpleAction::new("hide", None);
    action_hide.connect_activate(move |_, _| {
        if let Err(e) = db_hide.set_hidden(&id_hide, new_hidden_state) {
            eprintln!("Ошибка изменения видимости: {}", e);
        } else {
            let verb = if new_hidden_state { "скрыт" } else { "показан снова" };
            println!("[Меню] {}: {}", name_hide, verb);
            window_hide.rebuild_apps(&db_hide);
        }
    });

    action_group.add_action(&action_launch);
    action_group.add_action(&action_add_category);
    action_group.add_action(&action_pin);
    action_group.add_action(&action_hide);

    action_group
}