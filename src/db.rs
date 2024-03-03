use std::fs::create_dir_all;
use std::path::Path;

use conerror::conerror;
use sqlx::SqlitePool;

const DDL: &'static str = r#"
CREATE TABLE IF NOT EXISTS user (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    salt BLOB NOT NULL,
    credential BLOB NOT NULL,
    suspend INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS token (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    credential BLOB NOT NULL,
    last_active INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS index_token_user_id ON token(user_id);
CREATE TABLE IF NOT EXISTS password (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    username BLOB NOT NULL,
    password BLOB NOT NULL,
    attachment BLOB,
    updated_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS index_password_user_id ON password(user_id);
"#;

#[conerror]
pub async fn setup_db(data_dir: &str) -> conerror::Result<SqlitePool> {
    let path = Path::new(data_dir);
    if !path.exists() {
        create_dir_all(path)?;
    }
    let mut path = path.canonicalize()?;
    path.push("database");
    let db = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", path.to_str().unwrap())).await?;
    sqlx::query(DDL).execute(&db).await?;
    Ok(db)
}
