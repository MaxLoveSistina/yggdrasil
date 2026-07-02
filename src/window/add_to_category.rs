use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

/// Показывает popover: чекбоксы существующих категорий + Сохранить / Создать новую
pub fn show(parent_button: &gtk4::Button, window: &MainWindow, db: Rc<AppsDb>, app_id: String, app_name: String) {
    let popover = gtk4::Popover::new();
    popover.set_parent(parent_button);
    popover.set_position(gtk4::PositionType::Right);

    let popover_for_destroy = popover.clone();
    parent_button.connect_destroy(move |_| {
        popover_for_destroy.unparent();
    });

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_width_request(220);

    let title = gtk4::Label::new(Some(&format!("Categories for «{}»", app_name)));
    title.add_css_class("popover-title");
    content.append(&title);

    let categories = db.all_categories().unwrap_or_default();
    let dynamic: Vec<_> = categories.into_iter().filter(|c| !c.is_static).collect();
    let already_in: Vec<String> = db.categories_for_app(&app_id).unwrap_or_default();

    let checkboxes: Rc<RefCell<Vec<(String, gtk4::CheckButton)>>> = Rc::new(RefCell::new(Vec::new()));

    if dynamic.is_empty() {
        let placeholder = gtk4::Label::new(Some("No categories yet"));
        placeholder.set_margin_top(4);
        placeholder.set_margin_bottom(4);
        content.append(&placeholder);
    } else {
        for category in &dynamic {
            let check = gtk4::CheckButton::with_label(&category.name);
            check.set_active(already_in.contains(&category.name));
            content.append(&check);
            checkboxes.borrow_mut().push((category.name.clone(), check));
        }
    }

    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    content.append(&separator);

    let buttons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let create_btn = gtk4::Button::with_label("+");
    create_btn.add_css_class("dialog-button");
    create_btn.set_width_request(36);
    create_btn.set_height_request(36);
    create_btn.set_tooltip_text(Some("Create new category"));

    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    save_btn.set_hexpand(true);

    buttons_row.append(&save_btn);
    buttons_row.append(&create_btn);
    content.append(&buttons_row);

    popover.set_child(Some(&content));

    let db_for_save = db.clone();
    let app_id_for_save = app_id.clone();
    let popover_for_save = popover.clone();
    let checkboxes_for_save = checkboxes.clone();
    save_btn.connect_clicked(move |_| {
        for (name, check) in checkboxes_for_save.borrow().iter() {
            if check.is_active() {
                if let Err(e) = db_for_save.add_app_to_category(&app_id_for_save, name) {
                    eprintln!("Error adding to category «{}»: {}", name, e);
                }
            } else {
                if let Err(e) = db_for_save.remove_app_from_category(&app_id_for_save, name) {
                    eprintln!("Error removing from category «{}»: {}", name, e);
                }
            }
        }
        println!("Categories for «{}» saved", app_id_for_save);
        popover_for_save.popdown();
    });

    let window_clone = window.clone();
    let parent_button_clone = parent_button.clone();
    let popover_clone = popover.clone();
    let db_for_create = db.clone();
    let app_id_for_create = app_id.clone();
    let app_name_for_create = app_name.clone();
    create_btn.connect_clicked(move |_| {
        popover_clone.popdown();
        show_create_category_popover(
            &parent_button_clone,
            &window_clone,
            db_for_create.clone(),
            app_id_for_create.clone(),
            app_name_for_create.clone(),
        );
    });

    popover.popup();
}

fn show_create_category_popover(
    parent_button: &gtk4::Button,
    window: &MainWindow,
    db: Rc<AppsDb>,
    app_id: String,
    app_name: String,
) {
    let popover = gtk4::Popover::new();
    popover.set_parent(parent_button);
    popover.set_position(gtk4::PositionType::Right);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_width_request(220);

    let title = gtk4::Label::new(Some("New category"));
    title.add_css_class("popover-title");
    content.append(&title);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some("Category name"));
    content.append(&entry);

    let buttons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    buttons_row.set_halign(gtk4::Align::End);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("dialog-button");
    let add_btn = gtk4::Button::with_label("Create");
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
    let entry_clone = entry.clone();
    let window_clone = window.clone();
    let parent_button_clone = parent_button.clone();
    let app_id_clone = app_id.clone();
    let app_name_clone = app_name.clone();
    add_btn.connect_clicked(move |_| {
        let name = entry_clone.text().to_string();

        match db.add_category(&name) {
            Ok(true) => {
                println!("Category created: {}", name);

                crate::window::categories::populate(
                    &window_clone,
                    &db,
                    window_clone.current_category(),
                );

                popover_for_add.popdown();

                show(&parent_button_clone, &window_clone, db.clone(), app_id_clone.clone(), app_name_clone.clone());
            }
            Ok(false) => {
                println!("Could not create category (empty name or already exists): {}", name);
            }
            Err(e) => {
                eprintln!("Error creating category: {}", e);
            }
        }
    });

    let add_btn_clone = add_btn.clone();
    entry.connect_activate(move |_| {
        add_btn_clone.emit_clicked();
    });

    popover.popup();
}