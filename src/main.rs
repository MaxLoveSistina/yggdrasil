mod window;
mod apps_db;

use gtk4::prelude::*;
use gtk4::Application;
use std::rc::Rc;

const APP_ID: &str = "com.example.MainLauncher";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    load_css();

    let db = Rc::new(apps_db::AppsDb::open().expect("не удалось открыть БД"));
    db.sync_with_system().expect("ошибка синхронизации БД");

    let window = window::MainWindow::new(app);

    window.fullscreen();

    window.populate_apps(&db);
    window.present();
}

fn load_css() {
    let provider = gtk4::CssProvider::new();

    let css_path = find_css_path();

    match &css_path {
        Some(path) => {
            provider.load_from_path(path);
            println!("CSS загружен из: {}", path.display());
        }
        None => {
            eprintln!("style.css не найден, тема не применена");
        }
    }

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("нет дисплея"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Ищет style.css в порядке приоритета:
/// 1. src/resources/ рядом с бинарником (пользовательский)
/// 2. resources/ рядом с бинарником (дефолтный)
/// 3. resources/ в текущей директории (для cargo run)
fn find_css_path() -> Option<std::path::PathBuf> {
    // 1. Пользовательский CSS в src/resources/ рядом с бинарником
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let user_css = exe_dir.join("src/resources/style.css");
            if user_css.exists() {
                return Some(user_css);
            }
            
            // 2. Дефолтный CSS в resources/ рядом с бинарником
            let default_css = exe_dir.join("resources/style.css");
            if default_css.exists() {
                return Some(default_css);
            }
        }
    }

    // 3. Для разработки (cargo run) - ищем в текущей директории
    let cwd_css = std::path::PathBuf::from("resources/style.css");
    if cwd_css.exists() {
        return Some(cwd_css);
    }

    None
}