use gtk4::prelude::*;
use gtk4::{gio, gdk};
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;
use super::edit_dialog;

pub fn attach(
    button: &gtk4::Button,
    app: &gio::AppInfo,
    app_id: &str,
    app_name: &str,
    db: &Rc<AppsDb>,
    window: &MainWindow,
    is_hidden: bool,
    is_pinned: bool,
    is_manual: bool,
) {
    let hide_label = if is_hidden { "Unhide" } else { "Hide" };
    let pin_label = if is_pinned { "Unpin" } else { "Pin" };

    let menu_model = gio::Menu::new();
    menu_model.append(Some("Launch"), Some("app_card.launch"));
    menu_model.append(Some("Add to category"), Some("app_card.add_category"));
    menu_model.append(Some(pin_label), Some("app_card.pin"));
    menu_model.append(Some(hide_label), Some("app_card.hide"));
    
    // Edit только для ручных приложений
    if is_manual {
        menu_model.append(Some("Edit"), Some("app_card.edit"));
    }
    
    menu_model.append(Some("Delete"), Some("app_card.delete"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(button);

    let popover_for_destroy = popover.clone();
    button.connect_destroy(move |_| {
        popover_for_destroy.unparent();
    });

    let action_group = build_actions(button, app, app_id, app_name, db, window, is_hidden, is_pinned, is_manual);
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
    button: &gtk4::Button,
    app: &gio::AppInfo,
    app_id: &str,
    app_name: &str,
    db: &Rc<AppsDb>,
    window: &MainWindow,
    is_hidden: bool,
    is_pinned: bool,
    is_manual: bool,
) -> gio::SimpleActionGroup {
    let action_group = gio::SimpleActionGroup::new();

    // Launch
    let app_launch = app.clone();
    let name_launch = app_name.to_string();
    let window_launch = window.clone();
    let action_launch = gio::SimpleAction::new("launch", None);
    action_launch.connect_activate(move |_, _| {
        println!("[Меню] Launch: {}", name_launch);
        if let Err(e) = app_launch.launch(&[], gio::AppLaunchContext::NONE) {
            eprintln!("Не удалось запустить приложение: {}", e);
        } else {
            window_launch.close();
        }
    });

    // Add to category
    let id_category = app_id.to_string();
    let name_category = app_name.to_string();
    let db_category = db.clone();
    let window_category = window.clone();
    let button_for_category = button.clone();
    let action_add_category = gio::SimpleAction::new("add_category", None);
    action_add_category.connect_activate(move |_, _| {
        crate::window::add_to_category::show(
            &button_for_category, 
            &window_category, 
            db_category.clone(), 
            id_category.clone(), 
            name_category.clone()
        );
    });

    // Pin/Unpin
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

    // Hide/Unhide
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

    // Edit (только для ручных)
    if is_manual {
        let id_edit = app_id.to_string();
        let name_edit = app_name.to_string();
        let db_edit = db.clone();
        let window_edit = window.clone();
        let button_for_edit = button.clone();
        let action_edit = gio::SimpleAction::new("edit", None);
        action_edit.connect_activate(move |_, _| {
            edit_dialog::show(
                &button_for_edit, 
                &window_edit, 
                db_edit.clone(), 
                id_edit.clone(), 
                name_edit.clone()
            );
        });
        action_group.add_action(&action_edit);
    }

    action_group.add_action(&action_launch);
    action_group.add_action(&action_add_category);
    action_group.add_action(&action_pin);
    action_group.add_action(&action_hide);

    // Delete
    let id_delete = app_id.to_string();
    let db_delete = db.clone();
    let window_delete = window.clone();
    let is_manual_delete = is_manual;
    let action_delete = gio::SimpleAction::new("delete", None);
    action_delete.connect_activate(move |_, _| {
        if is_manual_delete {
            show_delete_options(&window_delete, db_delete.clone(), id_delete.clone());
        } else {
            crate::window::uninstall::handle_delete(&window_delete, db_delete.clone(), id_delete.clone());
        }
    });
    action_group.add_action(&action_delete);

    action_group
}

/// Диалог выбора типа удаления
fn show_delete_options(window: &MainWindow, db: Rc<AppsDb>, app_id: String) {
    let dialog = gtk4::AlertDialog::builder()
        .message("Delete Application")
        .detail("Choose deletion method:\n\n• Remove .desktop only: Removes launcher but keeps files\n• Full uninstall: Removes everything including save data")
        .buttons(["Cancel", "Remove .desktop only", "Full uninstall"])
        .cancel_button(0)
        .default_button(0)
        .build();

    let window_clone = window.clone();
    let db_clone = db.clone();
    let app_id_clone = app_id.clone();
    
    dialog.choose(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        match result {
            Ok(1) => {
                // Remove .desktop only
                remove_desktop_only(&app_id_clone, &db_clone);
                window_clone.rebuild_apps(&db_clone);
                window_clone.rebuild_pinned(&db_clone);
            }
            Ok(2) => {
                // Full uninstall
                crate::window::uninstall::handle_delete(
                    &window_clone, 
                    db_clone.clone(), 
                    app_id_clone.clone()
                );
            }
            _ => println!("Deletion cancelled"),
        }
    });
}

fn remove_desktop_only(app_id: &str, db: &Rc<AppsDb>) {
    let apps_dir = dirs::data_local_dir()
        .expect("no ~/.local/share")
        .join("applications");
    let desktop_path = apps_dir.join(app_id);
    
    if desktop_path.exists() {
        if let Err(e) = std::fs::remove_file(&desktop_path) {
            eprintln!("Error removing .desktop file: {}", e);
        } else {
            println!("Removed .desktop file: {}", app_id);
        }
    }
    
    if let Err(e) = db.remove_manual_app_record(app_id) {
        eprintln!("Error removing DB record: {}", e);
    }
}