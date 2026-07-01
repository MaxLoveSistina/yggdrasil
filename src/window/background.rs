use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use std::path::PathBuf;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

fn background_path() -> PathBuf {
    let mut path = AppsDb::config_dir();
    path.push("background.png");
    path
}

/// Показывает меню с пунктами "Сменить картинку" / "Убрать картинку", прикреплённое к кнопке
pub fn show_menu(button: &gtk4::Button, window: &MainWindow) {
    let menu_model = gtk4::gio::Menu::new();
    menu_model.append(Some("Сменить картинку"), Some("background.change"));
    menu_model.append(Some("Убрать картинку"), Some("background.remove"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(button);

    let popover_for_destroy = popover.clone();
    button.connect_destroy(move |_| {
        popover_for_destroy.unparent();
    });

    let action_group = gtk4::gio::SimpleActionGroup::new();

    let window_change = window.clone();
    let action_change = gtk4::gio::SimpleAction::new("change", None);
    action_change.connect_activate(move |_, _| {
        show_picker(&window_change);
    });

    let window_remove = window.clone();
    let action_remove = gtk4::gio::SimpleAction::new("remove", None);
    action_remove.connect_activate(move |_, _| {
        remove_background(&window_remove);
    });

    action_group.add_action(&action_change);
    action_group.add_action(&action_remove);
    button.insert_action_group("background", Some(&action_group));

    popover.popup();
}

/// Открывает системный диалог выбора изображения, копирует его в папку конфига и устанавливает как фон
fn show_picker(window: &MainWindow) {
    let filter = gtk4::FileFilter::new();
    filter.add_mime_type("image/*");
    filter.set_name(Some("Изображения"));

    let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter);

    let dialog = gtk4::FileDialog::builder()
        .title("Выберите фоновое изображение")
        .filters(&filters)
        .build();

    let window_clone = window.clone();
    dialog.open(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(source_path) = file.path() {
                match copy_and_apply(&window_clone, &source_path) {
                    Ok(_) => println!("Фон установлен и сохранён: {}", background_path().display()),
                    Err(e) => eprintln!("Ошибка установки фона: {}", e),
                }
            }
        }
    });
}

/// Копирует выбранный файл в папку конфига под фиксированным именем и применяет как фон
fn copy_and_apply(window: &MainWindow, source_path: &std::path::Path) -> std::io::Result<()> {
    let config_dir = AppsDb::config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let dest_path = background_path();
    std::fs::copy(source_path, &dest_path)?;

    let imp = window.imp();
    imp.background_picture.set_filename(Some(&dest_path));

    Ok(())
}

/// Удаляет сохранённый файл фона и очищает Picture
fn remove_background(window: &MainWindow) {
    let path = background_path();

    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            eprintln!("Ошибка удаления файла фона: {}", e);
            return;
        }
    }

    let imp = window.imp();
    imp.background_picture.set_filename(None::<PathBuf>);
    println!("Фон убран");
}

/// Вызывается при старте приложения — если ранее сохранённый фон существует, загружает его
pub fn load_saved(window: &MainWindow) {
    let path = background_path();

    if path.exists() {
        let imp = window.imp();
        imp.background_picture.set_filename(Some(&path));
        println!("Загружен сохранённый фон: {}", path.display());
    }
}