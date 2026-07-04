use gtk4::prelude::*;
use std::rc::Rc;
use std::path::PathBuf;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

pub fn show(
    parent_button: &gtk4::Button,
    window: &MainWindow,
    db: Rc<AppsDb>,
    app_id: String,
    app_name: String,
) {
    let popover = gtk4::Popover::new();
    popover.set_parent(parent_button);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    content.set_margin_top(14);
    content.set_margin_bottom(14);
    content.set_margin_start(14);
    content.set_margin_end(14);
    content.set_width_request(350);

    let title = gtk4::Label::new(Some("Edit Application"));
    title.add_css_class("popover-title");
    content.append(&title);

    // Имя
    let name_entry = create_name_section(&content, &app_name);
    
    // Иконка
    let (icon_path_label, new_icon_path) = create_icon_section(&content);
    
    // Путь
    let (exec_path_label, new_exec_path) = create_exec_section(&content);

    // Загружаем текущие данные
    if let Some((current_dir, _)) = db.get_manual_app(&app_id) {
        exec_path_label.set_text(&format!("Current: {}", current_dir));
    }

    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    content.append(&separator);

    // Кнопки
    let (cancel_btn, save_btn) = create_buttons(&content);

    popover.set_child(Some(&content));

    // Обработчики выбора файлов
    setup_icon_picker(&content, window, &popover, icon_path_label, new_icon_path.clone());
    setup_exec_picker(&content, window, &popover, exec_path_label, new_exec_path.clone());

    // Cancel
    let popover_for_cancel = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        popover_for_cancel.popdown();
    });

    // Save
    let popover_for_save = popover.clone();
    let window_clone = window.clone();
    let name_entry_clone = name_entry.clone();
    let app_id_clone = app_id.clone();
    let app_name_clone = app_name.clone();
    let new_icon_path_clone = new_icon_path.clone();
    let new_exec_path_clone = new_exec_path.clone();
    
    save_btn.connect_clicked(move |_| {
        save_changes(
            &app_id_clone,
            &app_name_clone,
            &name_entry_clone,
            &new_icon_path_clone,
            &new_exec_path_clone,
        );
        
        window_clone.rebuild_apps(&db);
        window_clone.rebuild_pinned(&db);
        popover_for_save.popdown();
    });

    popover.popup();
}

fn create_name_section(content: &gtk4::Box, app_name: &str) -> gtk4::Entry {
    let name_label = gtk4::Label::new(Some("Application name:"));
    name_label.set_halign(gtk4::Align::Start);
    content.append(&name_label);
    
    let name_entry = gtk4::Entry::new();
    name_entry.set_text(app_name);
    content.append(&name_entry);
    
    name_entry
}

fn create_icon_section(content: &gtk4::Box) -> (gtk4::Label, Rc<std::cell::RefCell<Option<PathBuf>>>) {
    let icon_label = gtk4::Label::new(Some("Icon:"));
    icon_label.set_halign(gtk4::Align::Start);
    content.append(&icon_label);
    
    let icon_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let icon_path_label = gtk4::Label::new(Some("Current icon"));
    icon_path_label.set_ellipsize(gtk4::pango::EllipsizeMode::Start);
    icon_path_label.set_hexpand(true);
    icon_path_label.set_halign(gtk4::Align::Start);
    
    let icon_btn = gtk4::Button::with_label("Browse");
    icon_btn.add_css_class("dialog-button");
    icon_btn.set_widget_name("icon-browse-btn");
    
    icon_row.append(&icon_path_label);
    icon_row.append(&icon_btn);
    content.append(&icon_row);
    
    let new_icon_path = Rc::new(std::cell::RefCell::new(None));
    (icon_path_label, new_icon_path)
}

fn create_exec_section(content: &gtk4::Box) -> (gtk4::Label, Rc<std::cell::RefCell<Option<PathBuf>>>) {
    let exec_label = gtk4::Label::new(Some("Executable path:"));
    exec_label.set_halign(gtk4::Align::Start);
    content.append(&exec_label);
    
    let exec_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let exec_path_label = gtk4::Label::new(Some("Current path"));
    exec_path_label.set_ellipsize(gtk4::pango::EllipsizeMode::Start);
    exec_path_label.set_hexpand(true);
    exec_path_label.set_halign(gtk4::Align::Start);
    
    let exec_btn = gtk4::Button::with_label("Browse");
    exec_btn.add_css_class("dialog-button");
    exec_btn.set_widget_name("exec-browse-btn");
    
    exec_row.append(&exec_path_label);
    exec_row.append(&exec_btn);
    content.append(&exec_row);
    
    let new_exec_path = Rc::new(std::cell::RefCell::new(None));
    (exec_path_label, new_exec_path)
}

fn create_buttons(content: &gtk4::Box) -> (gtk4::Button, gtk4::Button) {
    let buttons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    buttons_row.set_halign(gtk4::Align::End);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("dialog-button");
    
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");

    buttons_row.append(&cancel_btn);
    buttons_row.append(&save_btn);
    content.append(&buttons_row);
    
    (cancel_btn, save_btn)
}

fn setup_icon_picker(
    content: &gtk4::Box,
    window: &MainWindow,
    popover: &gtk4::Popover,
    icon_path_label: gtk4::Label,
    new_icon_path: Rc<std::cell::RefCell<Option<PathBuf>>>,
) {
    if let Some(icon_btn) = find_button_by_name(content, "icon-browse-btn") {
        let window_clone = window.clone();
        let popover_clone = popover.clone();
        let icon_label_clone = icon_path_label.clone();
        let icon_path_clone = new_icon_path.clone();
        
        icon_btn.connect_clicked(move |_| {
            popover_clone.popdown();
            
            let filter = gtk4::FileFilter::new();
            filter.add_mime_type("image/*");
            let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter);

            let dialog = gtk4::FileDialog::builder()
                .title("Select new icon")
                .filters(&filters)
                .build();

            let window_clone2 = window_clone.clone();
            let popover_clone2 = popover_clone.clone();
            let icon_label_clone2 = icon_label_clone.clone();
            let icon_path_clone2 = icon_path_clone.clone();
            
            dialog.open(Some(&window_clone2), gtk4::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        icon_label_clone2.set_text(&format!("New icon: {}", path.display()));
                        *icon_path_clone2.borrow_mut() = Some(path);
                    }
                }
                popover_clone2.popup();
            });
        });
    }
}

fn setup_exec_picker(
    content: &gtk4::Box,
    window: &MainWindow,
    popover: &gtk4::Popover,
    exec_path_label: gtk4::Label,
    new_exec_path: Rc<std::cell::RefCell<Option<PathBuf>>>,
) {
    if let Some(exec_btn) = find_button_by_name(content, "exec-browse-btn") {
        let window_clone = window.clone();
        let popover_clone = popover.clone();
        let exec_label_clone = exec_path_label.clone();
        let exec_path_clone = new_exec_path.clone();
        
        exec_btn.connect_clicked(move |_| {
            popover_clone.popdown();
            
            let dialog = gtk4::FileDialog::builder()
                .title("Select new executable")
                .build();

            let window_clone2 = window_clone.clone();
            let popover_clone2 = popover_clone.clone();
            let exec_label_clone2 = exec_label_clone.clone();
            let exec_path_clone2 = exec_path_clone.clone();
            
            dialog.open(Some(&window_clone2), gtk4::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        exec_label_clone2.set_text(&format!("New path: {}", path.display()));
                        *exec_path_clone2.borrow_mut() = Some(path);
                    }
                }
                popover_clone2.popup();
            });
        });
    }
}

fn find_button_by_name(container: &gtk4::Box, name: &str) -> Option<gtk4::Button> {
    let mut child = container.first_child();
    while let Some(widget) = child {
        if widget.widget_name() == name {
            // Исправление: клонируем widget перед downcast
            return widget.clone().downcast::<gtk4::Button>().ok();
        }
        child = widget.next_sibling();
    }
    None
}

fn save_changes(
    app_id: &str,
    old_name: &str,
    name_entry: &gtk4::Entry,
    new_icon_path: &Rc<std::cell::RefCell<Option<PathBuf>>>,
    new_exec_path: &Rc<std::cell::RefCell<Option<PathBuf>>>,
) {
    let new_name = name_entry.text().to_string();
    
    if new_name.trim().is_empty() {
        println!("Name cannot be empty");
        return;
    }

    if new_name != old_name {
        if let Err(e) = update_desktop_field(app_id, "Name", &new_name) {
            eprintln!("Error updating name: {}", e);
        }
    }

    if let Some(ref icon_path) = *new_icon_path.borrow() {
        if let Err(e) = update_desktop_field(app_id, "Icon", &icon_path.display().to_string()) {
            eprintln!("Error updating icon: {}", e);
        }
    }

    if let Some(ref exec_path) = *new_exec_path.borrow() {
        if let Err(e) = update_desktop_field(app_id, "Exec", &exec_path.display().to_string()) {
            eprintln!("Error updating executable path: {}", e);
        }
    }
}

fn update_desktop_field(app_id: &str, key: &str, new_value: &str) -> std::io::Result<()> {
    let apps_dir = dirs::data_local_dir()
        .expect("no ~/.local/share")
        .join("applications");
    let desktop_path = apps_dir.join(app_id);
    
    let content = std::fs::read_to_string(&desktop_path)?;
    let old_value = extract_desktop_value(&content, key);
    
    let new_content = if content.contains(&format!("{}=", key)) {
        content.replace(&format!("{}={}", key, old_value), &format!("{}={}", key, new_value))
    } else {
        content + &format!("\n{}={}", key, new_value)
    };
    
    std::fs::write(&desktop_path, new_content)?;
    Ok(())
}

fn extract_desktop_value(content: &str, key: &str) -> String {
    for line in content.lines() {
        if line.starts_with(&format!("{}=", key)) {
            return line.splitn(2, '=').nth(1).unwrap_or("").to_string();
        }
    }
    String::new()
}