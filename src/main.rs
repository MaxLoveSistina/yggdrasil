mod window;
mod apps_db;

use gtk4::prelude::*;
use gtk4::Application;
use gtk4_layer_shell::{LayerShell, Layer, KeyboardMode, Edge};
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

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_exclusive_zone(-1);

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

/// Ищет style.css рядом с исполняемым файлом, а если не найден — в текущей директории (для cargo run)
fn find_css_path() -> Option<std::path::PathBuf> {
    // 1. рядом с самим бинарником (для релизного запуска откуда угодно)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join("resources/style.css");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 2. текущая рабочая директория (удобно при cargo run во время разработки)
    let cwd_candidate = std::path::PathBuf::from("resources/style.css");
    if cwd_candidate.exists() {
        return Some(cwd_candidate);
    }

    None
}