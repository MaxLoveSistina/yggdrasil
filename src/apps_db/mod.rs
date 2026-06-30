use rusqlite::Connection;
use std::path::PathBuf;

pub struct AppsDb {
    conn: Connection,
}

impl AppsDb {
    /// Открывает (или создаёт) БД в ~/.config/MainLauncher/apps.db
    pub fn open() -> rusqlite::Result<Self> {
        let mut path: PathBuf = dirs::config_dir().expect("не удалось найти ~/.config");
        path.push("MainLauncher");
        std::fs::create_dir_all(&path).expect("не удалось создать папку конфига");
        path.push("apps.db");

        println!("БД находится тут: {}", path.display());

        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS apps (
                id           TEXT PRIMARY KEY,
                hidden       INTEGER NOT NULL DEFAULT 0,
                pinned       INTEGER NOT NULL DEFAULT 0,
                pinned_panel TEXT,
                collections  TEXT NOT NULL DEFAULT ''
            )",
            (),
        )?;

        Ok(Self { conn })
    }

    /// Добавляет приложение в БД, если его там ещё нет.
    /// Если уже есть — ничего не трогает (INSERT OR IGNORE).
    pub fn ensure_exists(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO apps (id) VALUES (?1)",
            [id],
        )?;
        Ok(())
    }

    /// Сканирует все .desktop приложения через gio и закидывает новые в БД.
    /// Уже существующие записи (с твоими pinned/hidden/collections) не трогаются.
    pub fn sync_with_system(&self) -> rusqlite::Result<()> {
        let apps = gtk4::gio::AppInfo::all();
        let mut added = 0;

        for app in &apps {
            if let Some(id) = gtk4::prelude::AppInfoExt::id(app) {
                let id = id.to_string();
                let before = self.count()?;
                self.ensure_exists(&id)?;
                let after = self.count()?;
                if after > before {
                    added += 1;
                }
            }
        }

        println!("Синхронизация завершена. Новых приложений добавлено: {}", added);
        Ok(())
    }

    fn count(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM apps", [], |row| row.get(0))
    }

    pub fn is_hidden(&self, id: &str) -> bool {
        self.conn
            .query_row(
                "SELECT hidden FROM apps WHERE id = ?1",
                [id],
                |row| row.get::<_, i32>(0),
            )
            .map(|v| v != 0)
            .unwrap_or(false)
    }

    pub fn set_hidden(&self, id: &str, hidden: bool) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE apps SET hidden = ?1 WHERE id = ?2",
            (hidden as i32, id),
        )?;
        Ok(())
    }

    pub fn set_pinned(&self, id: &str, pinned: bool) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE apps SET pinned = ?1 WHERE id = ?2",
            (pinned as i32, id),
        )?;
        Ok(())
    }

    /// Пример запроса "категория all, исключая hidden" — список id видимых приложений
    pub fn visible_ids(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT id FROM apps WHERE hidden = 0")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    /// Выводит всю БД в консоль (для дебага, как ты просил)
    pub fn print_all(&self) -> rusqlite::Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, hidden, pinned, pinned_panel, collections FROM apps")?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        println!("---- Содержимое БД ----");
        for row in rows {
            let (id, hidden, pinned, panel, collections) = row?;
            println!(
                "id: {:<40} hidden: {:<5} pinned: {:<5} panel: {:?} collections: {:?}",
                id,
                hidden != 0,
                pinned != 0,
                panel,
                collections
            );
        }
        println!("------------------------");

        Ok(())
    }
}