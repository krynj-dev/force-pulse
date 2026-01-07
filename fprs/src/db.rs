use rusqlite::{Connection, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn init_db(path: &Path) -> Result<Connection> {
    if path.exists() {
        return Connection::open(path);
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

    Ok(conn)
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

pub fn game_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM game", [], |row| row.get(0))
}

pub fn all_games(conn: &Connection) -> rusqlite::Result<Vec<(String, Value)>> {
    let mut stmt = conn.prepare(
        r#"
SELECT id, data FROM game
ORDER BY json_extract(game.data, "$.info.gameEndTimestamp") DESC"#,
    )?;

    let rows = stmt.query_map([], |row| {
        let data_str = row.get::<_, String>(1)?;
        Ok((
            row.get::<_, i64>(0)?.to_string(),
            serde_json::from_str(&data_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        ))
    });

    let items: Vec<(String, Value)> = rows?.collect::<Result<_>>()?;

    Ok(items)
}

pub fn game_by_id(conn: &Connection, id: u64) -> Result<Value> {
    let json_str: String =
        conn.query_row("SELECT data FROM game WHERE id=?1 LIMIT 1", [id], |row| {
            row.get(0)
        })?;

    let value: Value = serde_json::from_str(&json_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            json_str.len(),
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;

    Ok(value)
}
