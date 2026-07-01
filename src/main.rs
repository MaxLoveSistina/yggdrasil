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
    window.populate_apps(&db);
    window.present();
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_path("resources/style.css");

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("нет дисплея"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}