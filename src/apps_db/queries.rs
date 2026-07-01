use gtk4::prelude::AppInfoExt;
use rusqlite::Connection;

pub fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
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

    create_categories_table(conn)?;
    create_app_categories_table(conn)?;

    Ok(())
}

/// Добавляет приложение в БД, если его там ещё нет.
/// Уже существующие записи (с pinned/hidden/collections) не трогаются.
fn ensure_exists(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("INSERT OR IGNORE INTO apps (id) VALUES (?1)", [id])
}

/// Сканирует все .desktop приложения через gio и закидывает новые в БД.
pub fn sync_with_system(conn: &Connection) -> rusqlite::Result<()> {
    let apps = gtk4::gio::AppInfo::all();
    let mut added = 0;

    for app in &apps {
        if let Some(id) = app.id() {
            added += ensure_exists(conn, id.as_str())?;
        }
    }

    println!("Синхронизация завершена. Новых приложений добавлено: {}", added);
    Ok(())
}

pub fn is_hidden(conn: &Connection, id: &str) -> bool {
    conn.query_row(
        "SELECT hidden FROM apps WHERE id = ?1",
        [id],
        |row| row.get::<_, i32>(0),
    )
    .map(|v| v != 0)
    .unwrap_or(false)
}

pub fn set_hidden(conn: &Connection, id: &str, hidden: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE apps SET hidden = ?1 WHERE id = ?2",
        (hidden as i32, id),
    )?;
    Ok(())
}

pub fn set_pinned(conn: &Connection, id: &str, pinned: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE apps SET pinned = ?1 WHERE id = ?2",
        (pinned as i32, id),
    )?;
    Ok(())
}

/// "Категория all, исключая hidden" — список id видимых приложений
pub fn visible_ids(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM apps WHERE hidden = 0")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect()
}

/// Выводит всю БД в консоль (для дебага)
pub fn print_all(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt =
        conn.prepare("SELECT id, hidden, pinned, pinned_panel, collections FROM apps")?;

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

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub is_static: bool,
}

/// Создаёт таблицу категорий и гарантирует наличие статических (All, Hidden)
pub fn create_categories_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            name      TEXT NOT NULL UNIQUE,
            is_static INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO categories (name, is_static) VALUES ('All', 1)",
        (),
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO categories (name, is_static) VALUES ('Hidden', 1)",
        (),
    )?;

    Ok(())
}

/// Все категории: All всегда первая, Hidden всегда последняя, остальные по алфавиту между ними
pub fn all_categories(conn: &Connection) -> rusqlite::Result<Vec<Category>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, is_static FROM categories
         ORDER BY
            CASE name WHEN 'All' THEN 0 WHEN 'Hidden' THEN 2 ELSE 1 END,
            name ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            is_static: row.get::<_, i32>(2)? != 0,
        })
    })?;

    rows.collect()
}

/// Добавляет новую пользовательскую категорию. Возвращает Ok(false), если имя уже занято
pub fn add_category(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }

    let changed = conn.execute(
        "INSERT OR IGNORE INTO categories (name, is_static) VALUES (?1, 0)",
        [trimmed],
    )?;

    Ok(changed > 0)
}

/// Удаляет категорию, но только если она не статическая (защита на уровне SQL)
pub fn remove_category(conn: &Connection, name: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM categories WHERE name = ?1 AND is_static = 0",
        [name],
    )
}

pub fn is_pinned(conn: &Connection, id: &str) -> bool {
    conn.query_row(
        "SELECT pinned FROM apps WHERE id = ?1",
        [id],
        |row| row.get::<_, i32>(0),
    )
    .map(|v| v != 0)
    .unwrap_or(false)
}

/// Список id закреплённых приложений
pub fn pinned_ids(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM apps WHERE pinned = 1")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect()
}

/// Создаёт таблицу связей приложение-категория
pub fn create_app_categories_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_categories (
            app_id      TEXT NOT NULL,
            category_id INTEGER NOT NULL,
            PRIMARY KEY (app_id, category_id),
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
        )",
        (),
    )?;
    Ok(())
}

/// Добавляет приложение в категорию
pub fn add_app_to_category(conn: &Connection, app_id: &str, category_name: &str) -> rusqlite::Result<bool> {
    let category_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM categories WHERE name = ?1",
            [category_name],
            |row| row.get(0),
        )
        .ok();

    let Some(category_id) = category_id else {
        return Ok(false);
    };

    let changed = conn.execute(
        "INSERT OR IGNORE INTO app_categories (app_id, category_id) VALUES (?1, ?2)",
        (app_id, category_id),
    )?;

    Ok(changed > 0)
}

/// Проверяет, состоит ли приложение в конкретной категории
pub fn app_in_category(conn: &Connection, app_id: &str, category_name: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM app_categories ac
         JOIN categories c ON c.id = ac.category_id
         WHERE ac.app_id = ?1 AND c.name = ?2",
        (app_id, category_name),
        |_| Ok(()),
    )
    .is_ok()
}

/// Все id приложений в указанной категории (по имени категории)
pub fn app_ids_in_category(conn: &Connection, category_name: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT ac.app_id FROM app_categories ac
         JOIN categories c ON c.id = ac.category_id
         WHERE c.name = ?1",
    )?;
    let rows = stmt.query_map([category_name], |row| row.get::<_, String>(0))?;
    rows.collect()
}

pub fn remove_app_from_category(conn: &Connection, app_id: &str, category_name: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM app_categories
         WHERE app_id = ?1
         AND category_id = (SELECT id FROM categories WHERE name = ?2)",
        (app_id, category_name),
    )
}

pub fn categories_for_app(conn: &Connection, app_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT c.name FROM app_categories ac
         JOIN categories c ON c.id = ac.category_id
         WHERE ac.app_id = ?1",
    )?;
    let rows = stmt.query_map([app_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

pub fn rename_category(conn: &Connection, old_name: &str, new_name: &str) -> rusqlite::Result<bool> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() || old_name == "All" || old_name == "Hidden" {
        return Ok(false);
    }

    let changed = conn.execute(
        "UPDATE categories SET name = ?1 WHERE name = ?2 AND is_static = 0",
        (trimmed, old_name),
    )?;

    Ok(changed > 0)
}