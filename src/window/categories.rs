use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::{gio, gdk};
use std::cell::RefCell;
use std::rc::Rc;

use crate::apps_db::AppsDb;

/// Строит панель категорий заново из БД
pub fn populate(window: &super::MainWindow, db: &Rc<AppsDb>, current_category: Rc<RefCell<String>>) {
    let imp = window.imp();

    while let Some(child) = imp.categories_bar.first_child() {
        imp.categories_bar.remove(&child);
    }

    let categories = db.all_categories().unwrap_or_default();

    for category in categories {
        let btn = gtk4::Button::with_label(&category.name);
        btn.add_css_class("flat");
        btn.add_css_class("category-button");

        let name_clone = category.name.clone();
        let grid = imp.app_grid.clone();
        let current_for_click = current_category.clone();
        btn.connect_clicked(move |_| {
            println!("Категория выбрана: {}", name_clone);
            *current_for_click.borrow_mut() = name_clone.clone();
            grid.invalidate_filter();
        });

        // ПКМ на пользовательской категории — меню Переименовать / Удалить
        if !category.is_static {
            attach_category_menu(&btn, &category.name, db, window, current_category.clone());
        }

        imp.categories_bar.append(&btn);
    }

    let add_btn = gtk4::Button::with_label("+");
    add_btn.add_css_class("flat");
    add_btn.add_css_class("category-button");

    let window_clone = window.clone();
    let db_clone = db.clone();
    let current_for_add = current_category.clone();
    let add_btn_for_click = add_btn.clone();
    add_btn.connect_clicked(move |_| {
        show_add_category_dialog(&add_btn_for_click, &window_clone, db_clone.clone(), current_for_add.clone());
    });

    imp.categories_bar.append(&add_btn);
}

fn attach_category_menu(
    btn: &gtk4::Button,
    category_name: &str,
    db: &Rc<AppsDb>,
    window: &super::MainWindow,
    current_category: Rc<RefCell<String>>,
) {
    let menu_model = gio::Menu::new();
    menu_model.append(Some("Переименовать"), Some("category.rename"));
    menu_model.append(Some("Удалить"), Some("category.delete"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(btn);

    let popover_for_destroy = popover.clone();
    btn.connect_destroy(move |_| {
        popover_for_destroy.unparent();
    });

    let action_group = gio::SimpleActionGroup::new();

    // Переименовать
    let name_rename = category_name.to_string();
    let db_rename = db.clone();
    let window_rename = window.clone();
    let current_rename = current_category.clone();
    let btn_for_rename = btn.clone();
    let action_rename = gio::SimpleAction::new("rename", None);
    action_rename.connect_activate(move |_, _| {
        show_rename_dialog(&btn_for_rename, &window_rename, db_rename.clone(), name_rename.clone(), current_rename.clone());
    });

    // Удалить
    let name_delete = category_name.to_string();
    let db_delete = db.clone();
    let window_delete = window.clone();
    let current_delete = current_category.clone();
    let action_delete = gio::SimpleAction::new("delete", None);
    action_delete.connect_activate(move |_, _| {
        match db_delete.remove_category(&name_delete) {
            Ok(changed) if changed > 0 => {
                println!("Категория удалена: {}", name_delete);

                if *current_delete.borrow() == name_delete {
                    *current_delete.borrow_mut() = "All".to_string();
                }

                populate(&window_delete, &db_delete, current_delete.clone());
                window_delete.imp().app_grid.invalidate_filter();
            }
            Ok(_) => println!("Категория не удалена (возможно, статическая): {}", name_delete),
            Err(e) => eprintln!("Ошибка удаления категории: {}", e),
        }
    });

    action_group.add_action(&action_rename);
    action_group.add_action(&action_delete);
    btn.insert_action_group("category", Some(&action_group));

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(gdk::BUTTON_SECONDARY);

    let popover_clone = popover.clone();
    gesture.connect_pressed(move |gesture, _n_press, x, y| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        popover_clone.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover_clone.popup();
    });

    btn.add_controller(gesture);
}

fn show_rename_dialog(
    parent_button: &gtk4::Button,
    parent: &super::MainWindow,
    db: Rc<AppsDb>,
    old_name: String,
    current_category: Rc<RefCell<String>>,
) {
    let popover = gtk4::Popover::new();
    popover.set_parent(parent_button);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_width_request(220);

    let entry = gtk4::Entry::new();
    entry.set_text(&old_name);
    content.append(&entry);

    let buttons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    buttons_row.set_halign(gtk4::Align::End);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("dialog-button");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");

    buttons_row.append(&cancel_btn);
    buttons_row.append(&save_btn);
    content.append(&buttons_row);

    popover.set_child(Some(&content));

    let popover_for_cancel = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        popover_for_cancel.popdown();
    });

    let popover_for_save = popover.clone();
    let entry_clone = entry.clone();
    let parent_clone = parent.clone();
    let old_name_clone = old_name.clone();
    save_btn.connect_clicked(move |_| {
        let new_name = entry_clone.text().to_string();

        match db.rename_category(&old_name_clone, &new_name) {
            Ok(true) => {
                println!("Category renamed: {} -> {}", old_name_clone, new_name);

                if *current_category.borrow() == old_name_clone {
                    *current_category.borrow_mut() = new_name.clone();
                }

                populate(&parent_clone, &db, current_category.clone());
                popover_for_save.popdown();
            }
            Ok(false) => {
                println!("Could not rename (empty or conflict): {}", new_name);
            }
            Err(e) => {
                eprintln!("Error renaming category: {}", e);
            }
        }
    });

    let save_btn_clone = save_btn.clone();
    entry.connect_activate(move |_| {
        save_btn_clone.emit_clicked();
    });

    popover.popup();
}

fn show_add_category_dialog(
    parent_button: &gtk4::Button,
    parent: &super::MainWindow,
    db: Rc<AppsDb>,
    current_category: Rc<RefCell<String>>,
) {
    let popover = gtk4::Popover::new();
    popover.set_parent(parent_button);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_width_request(220);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some("Category name"));
    content.append(&entry);

    let buttons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    buttons_row.set_halign(gtk4::Align::End);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("dialog-button");
    let add_btn = gtk4::Button::with_label("Add");
    add_btn.add_css_class("suggested-action");

    buttons_row.append(&cancel_btn);
    buttons_row.append(&add_btn);
    content.append(&buttons_row);

    popover.set_child(Some(&content));

    let popover_for_cancel = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        popover_for_cancel.popdown();
    });

    let popover_for_add = popover.clone();
    let parent_clone = parent.clone();
    let entry_clone = entry.clone();
    add_btn.connect_clicked(move |_| {
        let name = entry_clone.text().to_string();
        match db.add_category(&name) {
            Ok(true) => {
                println!("Category added: {}", name);
                populate(&parent_clone, &db, current_category.clone());
                popover_for_add.popdown();
            }
            Ok(false) => {
                println!("Category already exists or empty name: {}", name);
            }
            Err(e) => {
                eprintln!("Error adding category: {}", e);
            }
        }
    });

    let add_btn_clone = add_btn.clone();
    entry.connect_activate(move |_| {
        add_btn_clone.emit_clicked();
    });

    popover.popup();
}