use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

/// Показывает диалог: чекбоксы существующих категорий + Сохранить / Создать новую категорию
pub fn show(parent: &MainWindow, db: Rc<AppsDb>, app_id: String, app_name: String) {
    let dialog = gtk4::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(&format!("Категории для «{}»", app_name))
        .default_width(300)
        .default_height(300)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);

    let categories = db.all_categories().unwrap_or_default();
    let dynamic: Vec<_> = categories.into_iter().filter(|c| !c.is_static).collect();
    let already_in: Vec<String> = db.categories_for_app(&app_id).unwrap_or_default();

    let checkboxes: Rc<RefCell<Vec<(String, gtk4::CheckButton)>>> = Rc::new(RefCell::new(Vec::new()));

    if dynamic.is_empty() {
        let placeholder = gtk4::Label::new(Some("Пока нет категорий"));
        placeholder.set_margin_top(8);
        placeholder.set_margin_bottom(8);
        content.append(&placeholder);
    } else {
        let list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

        for category in &dynamic {
            let check = gtk4::CheckButton::with_label(&category.name);
            check.set_active(already_in.contains(&category.name));
            list_box.append(&check);
            checkboxes.borrow_mut().push((category.name.clone(), check));
        }

        let scroller = gtk4::ScrolledWindow::new();
        scroller.set_vexpand(true);
        scroller.set_child(Some(&list_box));
        content.append(&scroller);
    }

    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    content.append(&separator);

    let buttons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let create_btn = gtk4::Button::with_label("+");
    create_btn.add_css_class("dialog-button");
    create_btn.set_width_request(36);
    create_btn.set_height_request(36);
    create_btn.set_tooltip_text(Some("Создать новую категорию"));

    let save_btn = gtk4::Button::with_label("Сохранить");
    save_btn.add_css_class("suggested-action");
    save_btn.set_hexpand(true);

    buttons_row.append(&save_btn);
    buttons_row.append(&create_btn);

    content.append(&buttons_row);

    dialog.set_child(Some(&content));
    
    crate::window::dialog_utils::close_on_escape(&dialog);

    let db_for_save = db.clone();
    let app_id_for_save = app_id.clone();
    let dialog_for_save = dialog.clone();
    let checkboxes_for_save = checkboxes.clone();
    save_btn.connect_clicked(move |_| {
        for (name, check) in checkboxes_for_save.borrow().iter() {
            if check.is_active() {
                if let Err(e) = db_for_save.add_app_to_category(&app_id_for_save, name) {
                    eprintln!("Ошибка добавления в категорию «{}»: {}", name, e);
                }
            } else {
                if let Err(e) = db_for_save.remove_app_from_category(&app_id_for_save, name) {
                    eprintln!("Ошибка удаления из категории «{}»: {}", name, e);
                }
            }
        }
        println!("Категории для «{}» сохранены", app_id_for_save);
        dialog_for_save.close();
    });

    let parent_clone = parent.clone();
    let dialog_clone = dialog.clone();
    let db_for_create = db.clone();
    let app_id_for_create = app_id.clone();
    let app_name_for_create = app_name.clone();
    create_btn.connect_clicked(move |_| {
        dialog_clone.close();
        show_create_category_dialog(
            &parent_clone,
            db_for_create.clone(),
            app_id_for_create.clone(),
            app_name_for_create.clone(),
        );
    });

    dialog.present();
}

fn show_create_category_dialog(parent: &MainWindow, db: Rc<AppsDb>, app_id: String, app_name: String) {
    let dialog = gtk4::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Новая категория")
        .default_width(300)
        .default_height(120)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some("Название категории"));
    content.append(&entry);

    let buttons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    buttons_row.set_halign(gtk4::Align::End);

    let cancel_btn = gtk4::Button::with_label("Отмена");
    cancel_btn.add_css_class("dialog-button");
    let add_btn = gtk4::Button::with_label("Создать");
    add_btn.add_css_class("suggested-action");

    buttons_row.append(&cancel_btn);
    buttons_row.append(&add_btn);
    content.append(&buttons_row);

    dialog.set_child(Some(&content));

    let dialog_clone = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let entry_clone = entry.clone();
    let parent_clone = parent.clone();
    let app_id_clone = app_id.clone();
    let app_name_clone = app_name.clone();
    add_btn.connect_clicked(move |_| {
        let name = entry_clone.text().to_string();

        match db.add_category(&name) {
            Ok(true) => {
                println!("Категория создана: {}", name);

                crate::window::categories::populate(
                    &parent_clone,
                    &db,
                    Rc::new(RefCell::new("All".to_string())),
                );

                dialog_clone.close();

                show(&parent_clone, db.clone(), app_id_clone.clone(), app_name_clone.clone());
            }
            Ok(false) => {
                println!("Категория уже существует или имя пустое: {}", name);
            }
            Err(e) => {
                eprintln!("Ошибка создания категории: {}", e);
            }
        }
    });

    let add_btn_clone = add_btn.clone();
    entry.connect_activate(move |_| {
        add_btn_clone.emit_clicked();
    });

    dialog.present();
}