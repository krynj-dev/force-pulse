use rusqlite::{Connection, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn init_db(path: &Path) -> Result<()> {
    if path.exists() {
        return Err(rusqlite::Error::InvalidPath(path.to_path_buf()));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create config directory");
    }

    let conn = Connection::open(path)?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS game (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            data TEXT CHECK (json_valid(data)),
            manual INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )?;

    Ok(())
}

pub fn insert_game(conn: &Connection, game_id: &str, json: &Value) -> Result<()> {
    conn.execute(
        r#"
    INSERT INTO game (id, data)
    VALUES (?1, ?2)
    "#,
        (game_id, json.to_string()),
    )?;
    Ok(())
}

pub fn game_kda(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        SELECT
            json_extract(p.value, '$.puuid') AS puuid,
            json_extract(p.value, '$.riotIdGameName') AS name,
            json_extract(p.value, '$.championName')  AS champion,
            SUM(json_extract(p.value, '$.kills'))   AS total_kills,
            SUM(json_extract(p.value, '$.deaths'))  AS total_deaths,
            SUM(json_extract(p.value, '$.assists')) AS total_assists,
            ROUND(
                (SUM(json_extract(p.value, '$.kills')) +
                 SUM(json_extract(p.value, '$.assists'))) * 1.0 /
                CASE
                    WHEN SUM(json_extract(p.value, '$.deaths')) = 0 THEN 1
                    ELSE SUM(json_extract(p.value, '$.deaths'))
                END,
                2
            ) AS kda
        FROM game m
        JOIN json_each(m.match_json, '$.info.participants') p
        GROUP BY puuid
        ORDER BY kda DESC;
        "#,
        (),
    )?;
    Ok(())
}
