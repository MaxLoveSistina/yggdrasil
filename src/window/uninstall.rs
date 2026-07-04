use std::process::Command;
use std::rc::Rc;

use crate::apps_db::AppsDb;
use crate::window::MainWindow;

/// Точка входа: определяет тип приложения и выполняет соответствующее удаление
pub fn handle_delete(window: &MainWindow, db: Rc<AppsDb>, desktop_id: String) {
    // Проверяем, является ли приложение ручным (созданным через наш лаунчер)
    if desktop_id.starts_with("yggdrasil-manual-") {
        show_manual_confirm(window, db, desktop_id);
        return;
    }
    
    // Проверяем системный путь
    let system_path = format!("/usr/share/applications/{}", desktop_id);
    let system_desktop = std::path::Path::new(&system_path);
    
    if system_desktop.exists() {
        // Это системное приложение, пытаемся найти пакет
        match find_owning_package(&system_path) {
            Some(pkg) => {
                // Нашли пакет - удаляем через pacman
                show_package_confirm(window, db, desktop_id, pkg);
            }
            None => {
                // Пакет не найден - возможно, .desktop файл создан вручную
                // или установлен не через пакетный менеджер
                show_orphan_desktop_confirm(window, db, desktop_id, system_path);
            }
        }
    } else {
        // Проверяем пользовательский путь
        let user_path = dirs::data_local_dir()
            .expect("no ~/.local/share")
            .join("applications")
            .join(&desktop_id);
        
        if user_path.exists() {
            // Пользовательский .desktop файл
            show_user_desktop_confirm(window, db, desktop_id, user_path);
        } else {
            eprintln!("Cannot find .desktop file for: {}", desktop_id);
        }
    }
}

/// Подтверждение удаления системного .desktop файла без пакета
fn show_orphan_desktop_confirm(
    window: &MainWindow, 
    db: Rc<AppsDb>, 
    desktop_id: String,
    system_path: String
) {
    let dialog = gtk4::AlertDialog::builder()
        .message("Remove Application Entry")
        .detail(format!(
            "This .desktop file exists but doesn't belong to any package:\n{}\n\n\
             It might have been created manually or leftover from an incomplete uninstall.\n\n\
             Remove only the .desktop launcher file?",
            system_path
        ))
        .buttons(["Cancel", "Remove .desktop file"])
        .cancel_button(0)
        .default_button(0)
        .build();

    let window_clone = window.clone();
    let db_clone = db.clone();
    let _desktop_id_clone = desktop_id.clone();
    
    dialog.choose(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        if let Ok(1) = result {
            // Удаляем системный .desktop файл (потребуются права root)
            let status = Command::new("pkexec")
                .args(["rm", &system_path])
                .status();
            
            match status {
                Ok(s) if s.success() => {
                    println!("Removed .desktop file: {}", system_path);
                    window_clone.rebuild_apps(&db_clone);
                    window_clone.rebuild_pinned(&db_clone);
                }
                Ok(_) => eprintln!("Failed to remove .desktop file"),
                Err(e) => eprintln!("Error running pkexec: {}", e),
            }
        }
    });
}

/// Подтверждение удаления пользовательского .desktop файла
fn show_user_desktop_confirm(
    window: &MainWindow, 
    db: Rc<AppsDb>, 
    desktop_id: String,
    user_path: std::path::PathBuf
) {
    let dialog = gtk4::AlertDialog::builder()
        .message("Remove Application Entry")
        .detail(format!(
            "Remove this application launcher?\n\n\
             File: {}\n\n\
             This will only remove the .desktop file, not the application itself.",
            user_path.display()
        ))
        .buttons(["Cancel", "Remove"])
        .cancel_button(0)
        .default_button(0)
        .build();

    let window_clone = window.clone();
    let db_clone = db.clone();
    let desktop_id_clone = desktop_id.clone();
    let user_path_clone = user_path.clone();
    
    dialog.choose(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        if let Ok(1) = result {
            if let Err(e) = std::fs::remove_file(&user_path_clone) {
                eprintln!("Error removing .desktop file: {}", e);
            } else {
                println!("Removed .desktop file: {}", user_path_clone.display());
                
                // Если была запись в БД - удаляем и её
                if desktop_id_clone.starts_with("yggdrasil-manual-") {
                    if let Err(e) = db_clone.remove_manual_app_record(&desktop_id_clone) {
                        eprintln!("Error removing DB record: {}", e);
                    }
                }
                
                window_clone.rebuild_apps(&db_clone);
                window_clone.rebuild_pinned(&db_clone);
            }
        }
    });
}

/// Полное удаление вручную добавленного приложения: .desktop файл + вся папка приложения
fn show_manual_confirm(window: &MainWindow, db: Rc<AppsDb>, desktop_id: String) {
    let app_dir = db.get_manual_app(&desktop_id).map(|(dir, _)| dir);

    let dialog = gtk4::AlertDialog::builder()
        .message("Delete Application")
        .detail(match &app_dir {
            Some(dir) => format!(
                "This will permanently delete the application folder:\n{}\n\n\
                 This includes all save data stored inside it. This cannot be undone.\n\n\
                 Choose 'Remove .desktop only' to keep the application files.",
                dir
            ),
            None => "This will remove the application from the launcher.".to_string(),
        })
        .buttons(["Cancel", "Remove .desktop only", "Full uninstall"])
        .cancel_button(0)
        .default_button(0)
        .build();

    let window_clone = window.clone();
    let db_clone = db.clone();
    let desktop_id_clone = desktop_id.clone();
    
    dialog.choose(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        match result {
            Ok(1) => {
                // Remove .desktop only
                remove_desktop_file(&desktop_id_clone, &db_clone);
                window_clone.rebuild_apps(&db_clone);
                window_clone.rebuild_pinned(&db_clone);
            }
            Ok(2) => {
                // Full uninstall
                perform_manual_delete(&window_clone, &db_clone, &desktop_id_clone);
            }
            _ => println!("Deletion cancelled"),
        }
    });
}

fn perform_manual_delete(window: &MainWindow, db: &Rc<AppsDb>, desktop_id: &str) {
    // 1. удаляем всю папку приложения
    if let Some((app_dir, _extra)) = db.get_manual_app(desktop_id) {
        let path = std::path::PathBuf::from(&app_dir);
        if path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&path) {
                eprintln!("Error removing application folder {}: {}", app_dir, e);
            } else {
                println!("Removed application folder: {}", app_dir);
            }
        }
    }

    // 2. удаляем .desktop файл и запись в БД
    remove_desktop_file(desktop_id, db);
    
    // 3. обновляем интерфейс
    println!("Application fully removed: {}", desktop_id);
    window.rebuild_apps(db);
    window.rebuild_pinned(db);
}

/// Удаляет только .desktop файл и запись в БД
fn remove_desktop_file(desktop_id: &str, db: &Rc<AppsDb>) {
    let apps_dir = dirs::data_local_dir()
        .expect("no ~/.local/share")
        .join("applications");
    let desktop_path = apps_dir.join(desktop_id);
    
    if desktop_path.exists() {
        if let Err(e) = std::fs::remove_file(&desktop_path) {
            eprintln!("Error removing .desktop file: {}", e);
        } else {
            println!("Removed .desktop file: {}", desktop_id);
        }
    }
    
    // Удаляем запись из БД если есть
    if let Err(_e) = db.remove_manual_app_record(desktop_id) {
        // Не ошибка, если записи не было
        println!("No DB record to remove for: {}", desktop_id);
    }
}

/// Удаление пакетного приложения через pacman
fn show_package_confirm(window: &MainWindow, db: Rc<AppsDb>, _desktop_id: String, package_name: String) {
    let dialog = gtk4::AlertDialog::builder()
        .message("Uninstall Package")
        .detail(format!(
            "This will run: pacman -Rns {}\n\n\
             This removes the package and its unused dependencies.\n\
             You will be asked for your password.",
            package_name
        ))
        .buttons(["Cancel", "Uninstall"])
        .cancel_button(0)
        .default_button(0)
        .build();

    let window_clone = window.clone();
    let db_clone = db.clone();
    
    dialog.choose(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        if let Ok(1) = result {
            perform_package_uninstall(&window_clone, &db_clone, &package_name);
        }
    });
}

fn find_owning_package(desktop_path: &str) -> Option<String> {
    let output = Command::new("pacman").args(["-Qo", desktop_path]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split("is owned by")
        .nth(1)?
        .split_whitespace()
        .next()
        .map(String::from)
}

fn perform_package_uninstall(window: &MainWindow, db: &Rc<AppsDb>, package_name: &str) {
    let status = Command::new("pkexec")
        .args(["pacman", "-Rns", "--noconfirm", package_name])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Package uninstalled: {}", package_name);
            window.rebuild_apps(db);
            window.rebuild_pinned(db);
        }
        Ok(_) => println!("Uninstall cancelled or failed"),
        Err(e) => eprintln!("Error running pkexec: {}", e),
    }
}