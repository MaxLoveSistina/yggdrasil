mod window;
mod apps_db;

use gtk4::prelude::*;
use gtk4::Application;

const APP_ID: &str = "com.example.MainLauncher";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // 1. Открываем (или создаём) БД
    let db = apps_db::AppsDb::open().expect("не удалось открыть БД");

    // 2. Сканируем систему и добавляем новые приложения
    db.sync_with_system().expect("ошибка синхронизации БД");

    // 3. Выводим всю БД в консоль
    db.print_all().expect("ошибка чтения БД");

    // 4. Запускаем графическую часть
    let window = window::MainWindow::new(app);
    window.populate_apps(&db);
    window.present();
}