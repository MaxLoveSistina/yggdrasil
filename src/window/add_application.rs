use gtk4::prelude::*;
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

/// Показывает popover с формой добавления приложения вручную
pub fn show(parent_button: &gtk4::Button, window: &MainWindow, db: Rc<AppsDb>) {
    let popover = gtk4::Popover::new();
    popover.set_parent(parent_button);
    popover.set_autohide(false); // Отключаем автоматическое скрытие

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    content.set_margin_top(14);
    content.set_margin_bottom(14);
    content.set_margin_start(14);
    content.set_margin_end(14);
    content.set_width_request(320);

    let title = gtk4::Label::new(Some("Add Application"));
    title.add_css_class("popover-title");
    content.append(&title);

    // Название приложения
    let name_entry = gtk4::Entry::new();
    name_entry.set_placeholder_text(Some("Application name"));
    content.append(&name_entry);

    // Путь к исполняемому файлу
    let exec_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let exec_label = gtk4::Label::new(Some("Executable: not selected"));
    exec_label.set_ellipsize(gtk4::pango::EllipsizeMode::Start);
    exec_label.set_hexpand(true);
    exec_label.set_halign(gtk4::Align::Start);
    let exec_btn = gtk4::Button::with_label("Browse");
    exec_btn.add_css_class("dialog-button");
    exec_row.append(&exec_label);
    exec_row.append(&exec_btn);
    content.append(&exec_row);

    // Путь к иконке (опционально)
    let icon_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let icon_label = gtk4::Label::new(Some("Icon: default"));
    icon_label.set_ellipsize(gtk4::pango::EllipsizeMode::Start);
    icon_label.set_hexpand(true);
    icon_label.set_halign(gtk4::Align::Start);
    let icon_btn = gtk4::Button::with_label("Browse");
    icon_btn.add_css_class("dialog-button");
    icon_row.append(&icon_label);
    icon_row.append(&icon_btn);
    content.append(&icon_row);

    let hint = gtk4::Label::new(Some(
        "Tip: place the game in its own folder — Yggdrasil will remember\nthis folder so it can be fully removed later, including save data."
    ));
    hint.set_wrap(true);
    hint.add_css_class("hint-text");
    content.append(&hint);

    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    content.append(&separator);

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

    // храним выбранные пути в замыканиях через Rc<RefCell<...>>
    let exec_path: Rc<std::cell::RefCell<Option<std::path::PathBuf>>> = Rc::new(std::cell::RefCell::new(None));
    let icon_path: Rc<std::cell::RefCell<Option<std::path::PathBuf>>> = Rc::new(std::cell::RefCell::new(None));

    // выбор исполняемого файла
    let exec_path_for_pick = exec_path.clone();
    let exec_label_clone = exec_label.clone();
    let popover_for_exec = popover.clone();
    let window_for_exec = window.clone();
    exec_btn.connect_clicked(move |_| {
        // Скрываем popover перед открытием диалога
        popover_for_exec.popdown();
        
        let dialog = gtk4::FileDialog::builder()
            .title("Select executable file")
            .build();

        let exec_path_clone = exec_path_for_pick.clone();
        let exec_label_clone2 = exec_label_clone.clone();
        let window_clone = window_for_exec.clone();
        let popover_clone = popover_for_exec.clone();
        
        dialog.open(Some(&window_clone), gtk4::gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    exec_label_clone2.set_text(&format!("Executable: {}", path.display()));
                    *exec_path_clone.borrow_mut() = Some(path);
                }
            }
            // После выбора файла показываем popover снова
            popover_clone.popup();
        });
    });

    // выбор иконки
    let icon_path_for_pick = icon_path.clone();
    let icon_label_clone = icon_label.clone();
    let popover_for_icon = popover.clone();
    let window_for_icon = window.clone();
    icon_btn.connect_clicked(move |_| {
        // Скрываем popover перед открытием диалога
        popover_for_icon.popdown();
        
        let filter = gtk4::FileFilter::new();
        filter.add_mime_type("image/*");
        let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);

        let dialog = gtk4::FileDialog::builder()
            .title("Select icon (optional)")
            .filters(&filters)
            .build();

        let icon_path_clone = icon_path_for_pick.clone();
        let icon_label_clone2 = icon_label_clone.clone();
        let window_clone = window_for_icon.clone();
        let popover_clone = popover_for_icon.clone();
        
        dialog.open(Some(&window_clone), gtk4::gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    icon_label_clone2.set_text(&format!("Icon: {}", path.display()));
                    *icon_path_clone.borrow_mut() = Some(path);
                }
            }
            // После выбора файла показываем popover снова
            popover_clone.popup();
        });
    });

    let popover_for_cancel = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        popover_for_cancel.popdown();
    });

    let popover_for_add = popover.clone();
    let window_clone = window.clone();
    let name_entry_clone = name_entry.clone();
    add_btn.connect_clicked(move |_| {
        let name = name_entry_clone.text().to_string();

        if name.trim().is_empty() {
            println!("Enter an application name");
            return;
        }

        let exec = match exec_path.borrow().clone() {
            Some(p) => p,
            None => {
                println!("Select an executable file first");
                return;
            }
        };

        let icon = icon_path.borrow().clone();

        match create_manual_app(&db, &name, &exec, icon.as_deref()) {
            Ok(desktop_id) => {
                println!("Application added: {} ({})", name, desktop_id);
                window_clone.rebuild_apps(&db);
                popover_for_add.popdown();
            }
            Err(e) => {
                eprintln!("Error adding application: {}", e);
            }
        }
    });

    popover.popup();
}

/// Создаёт .desktop файл в ~/.local/share/applications и записывает метаданные для последующего удаления
fn create_manual_app(
    db: &AppsDb,
    name: &str,
    exec_path: &std::path::Path,
    icon_path: Option<&std::path::Path>,
) -> std::io::Result<String> {
    let apps_dir = dirs::data_local_dir()
        .expect("no ~/.local/share")
        .join("applications");
    std::fs::create_dir_all(&apps_dir)?;

    let safe_name: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let desktop_id = format!("yggdrasil-manual-{}.desktop", safe_name);
    let desktop_path = apps_dir.join(&desktop_id);

    let icon_line = icon_path
        .map(|p| format!("Icon={}\n", p.display()))
        .unwrap_or_default();

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Exec=\"{}\"\n\
         {icon_line}\
         Categories=Game;\n\
         Terminal=false\n",
        name,
        exec_path.display()
    );

    std::fs::write(&desktop_path, content)?;

    // запоминаем папку приложения (родительскую директорию исполняемого файла)
    // — именно её будем полностью удалять при uninstall
    let app_dir = exec_path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    db.add_manual_app(&desktop_id, &app_dir, "")
        .expect("failed to save manual app record");

    Ok(desktop_id)
}