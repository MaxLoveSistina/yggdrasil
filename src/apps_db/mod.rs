mod queries;

use rusqlite::Connection;
use std::path::PathBuf;

pub use queries::Category;

pub struct AppsDb {
    conn: Connection,
}

impl AppsDb {
    pub fn open() -> rusqlite::Result<Self> {
        let mut path: PathBuf = dirs::config_dir().expect("не удалось найти ~/.config");
        path.push("MainLauncher");
        std::fs::create_dir_all(&path).expect("не удалось создать папку конфига");
        path.push("apps.db");

        println!("БД находится тут: {}", path.display());

        let conn = Connection::open(path)?;
        queries::create_schema(&conn)?;

        Ok(Self { conn })
    }

    pub fn sync_with_system(&self) -> rusqlite::Result<()> {
        queries::sync_with_system(&self.conn)
    }

    pub fn is_hidden(&self, id: &str) -> bool {
        queries::is_hidden(&self.conn, id)
    }

    pub fn set_hidden(&self, id: &str, hidden: bool) -> rusqlite::Result<()> {
        queries::set_hidden(&self.conn, id, hidden)
    }

    pub fn set_pinned(&self, id: &str, pinned: bool) -> rusqlite::Result<()> {
        queries::set_pinned(&self.conn, id, pinned)
    }

    pub fn visible_ids(&self) -> rusqlite::Result<Vec<String>> {
        queries::visible_ids(&self.conn)
    }

    pub fn print_all(&self) -> rusqlite::Result<()> {
        queries::print_all(&self.conn)
    }

    pub fn all_categories(&self) -> rusqlite::Result<Vec<Category>> {
        queries::all_categories(&self.conn)
    }

    pub fn add_category(&self, name: &str) -> rusqlite::Result<bool> {
        queries::add_category(&self.conn, name)
    }

    pub fn remove_category(&self, name: &str) -> rusqlite::Result<usize> {
        queries::remove_category(&self.conn, name)
    }

    pub fn is_pinned(&self, id: &str) -> bool {
        queries::is_pinned(&self.conn, id)
    }

    pub fn pinned_ids(&self) -> rusqlite::Result<Vec<String>> {
        queries::pinned_ids(&self.conn)
    }

    pub fn add_app_to_category(&self, app_id: &str, category_name: &str) -> rusqlite::Result<bool> {
    queries::add_app_to_category(&self.conn, app_id, category_name)
}

    pub fn app_ids_in_category(&self, category_name: &str) -> rusqlite::Result<Vec<String>> {
        queries::app_ids_in_category(&self.conn, category_name)
    }

    pub fn app_in_category(&self, app_id: &str, category_name: &str) -> bool {
        queries::app_in_category(&self.conn, app_id, category_name)
    }

    pub fn remove_app_from_category(&self, app_id: &str, category_name: &str) -> rusqlite::Result<usize> {
        queries::remove_app_from_category(&self.conn, app_id, category_name)
    }

    pub fn categories_for_app(&self, app_id: &str) -> rusqlite::Result<Vec<String>> {
        queries::categories_for_app(&self.conn, app_id)
    }

    pub fn config_dir() -> PathBuf {
        let mut path = dirs::config_dir().expect("не удалось найти ~/.config");
        path.push("MainLauncher");
        path
    }

    pub fn rename_category(&self, old_name: &str, new_name: &str) -> rusqlite::Result<bool> {
        queries::rename_category(&self.conn, old_name, new_name)
    }

    pub fn add_manual_app(&self, desktop_id: &str, app_dir: &str, extra_paths: &str) -> rusqlite::Result<()> {
        queries::add_manual_app(&self.conn, desktop_id, app_dir, extra_paths)
    }

    pub fn get_manual_app(&self, desktop_id: &str) -> Option<(String, String)> {
        queries::get_manual_app(&self.conn, desktop_id)
    }

    pub fn remove_manual_app_record(&self, desktop_id: &str) -> rusqlite::Result<()> {
        queries::remove_manual_app_record(&self.conn, desktop_id)
    }
}