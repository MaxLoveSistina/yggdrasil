use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use std::path::PathBuf;
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

fn background_path() -> PathBuf {
    let mut path = AppsDb::config_dir();
    path.push("background.png");
    path
}

fn default_background_path() -> Option<PathBuf> {
    // Ищем рядом с бинарником ВСЕ возможные пути
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let paths = vec![
                exe_dir.join("src/resources/default_bg.png"),
                exe_dir.join("resources/default_bg.png"),
                exe_dir.join("../resources/default_bg.png"),
                exe_dir.join("../../resources/default_bg.png"),
            ];
            
            for path in paths {
                if path.exists() {
                    println!("Found default bg: {}", path.display());
                    return Some(path);
                }
            }
        }
    }
    
    // Для cargo run
    let cwd_path = std::path::PathBuf::from("resources/default_bg.png");
    if cwd_path.exists() {
        return Some(cwd_path);
    }
    
    None
}

/// Shows the settings menu attached to the settings button
pub fn show_menu(button: &gtk4::Button, window: &MainWindow, db: Rc<AppsDb>) {
    let menu_model = gtk4::gio::Menu::new();
    menu_model.append(Some("Add Application"), Some("settings.add_app"));
    menu_model.append(Some("Change Background"), Some("settings.change_bg"));
    menu_model.append(Some("Remove Background"), Some("settings.remove_bg"));
    menu_model.append(Some("Quit"), Some("settings.quit"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(button);

    let popover_for_destroy = popover.clone();
    button.connect_destroy(move |_| {
        popover_for_destroy.unparent();
    });

    let action_group = gtk4::gio::SimpleActionGroup::new();

    let button_for_add = button.clone();
    let window_for_add = window.clone();
    let db_for_add = db.clone();
    let action_add = gtk4::gio::SimpleAction::new("add_app", None);
    action_add.connect_activate(move |_, _| {
        crate::window::add_application::show(&button_for_add, &window_for_add, db_for_add.clone());
    });

    let window_change = window.clone();
    let action_change = gtk4::gio::SimpleAction::new("change_bg", None);
    action_change.connect_activate(move |_, _| {
        show_picker(&window_change);
    });

    let window_remove = window.clone();
    let action_remove = gtk4::gio::SimpleAction::new("remove_bg", None);
    action_remove.connect_activate(move |_, _| {
        remove_background(&window_remove);
    });

    let window_quit = window.clone();
    let action_quit = gtk4::gio::SimpleAction::new("quit", None);
    action_quit.connect_activate(move |_, _| {
        window_quit.close();
    });

    action_group.add_action(&action_add);
    action_group.add_action(&action_change);
    action_group.add_action(&action_remove);
    action_group.add_action(&action_quit);
    button.insert_action_group("settings", Some(&action_group));

    popover.popup();
}

fn show_picker(window: &MainWindow) {
    let filter = gtk4::FileFilter::new();
    filter.add_mime_type("image/*");
    filter.set_name(Some("Images"));

    let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter);

    let dialog = gtk4::FileDialog::builder()
        .title("Select background image")
        .filters(&filters)
        .build();

    let window_clone = window.clone();
    dialog.open(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(source_path) = file.path() {
                match copy_and_apply(&window_clone, &source_path) {
                    Ok(_) => println!("Background set and saved: {}", background_path().display()),
                    Err(e) => eprintln!("Error setting background: {}", e),
                }
            }
        }
    });
}

fn copy_and_apply(window: &MainWindow, source_path: &std::path::Path) -> std::io::Result<()> {
    let config_dir = AppsDb::config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let dest_path = background_path();
    std::fs::copy(source_path, &dest_path)?;

    let imp = window.imp();
    imp.background_picture.set_filename(Some(&dest_path));

    Ok(())
}

fn remove_background(window: &MainWindow) {
    let path = background_path();

    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            eprintln!("Error removing background file: {}", e);
            return;
        }
    }

    let imp = window.imp();
    imp.background_picture.set_filename(None::<PathBuf>);
    println!("Background removed");
    
    // После удаления применяем дефолтный фон
    if let Some(default_bg) = default_background_path() {
        imp.background_picture.set_filename(Some(&default_bg));
        println!("Applied default background");
    }
}

pub fn load_saved(window: &MainWindow) {
    let path = background_path();

    if path.exists() {
        let imp = window.imp();
        imp.background_picture.set_filename(Some(&path));
        println!("Loaded saved background: {}", path.display());
    } else {
        // Если нет сохраненного - грузим дефолтный
        if let Some(default_bg) = default_background_path() {
            let imp = window.imp();
            imp.background_picture.set_filename(Some(&default_bg));
            println!("Applied default background");
        }
    }
}