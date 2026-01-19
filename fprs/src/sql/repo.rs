use crate::sql::schema::{
    self, ChampionHistory, ChampionStats, Game, OverallStats, PlayerChampionStats,
    PlayerOverallStats, PlayerRoleStats, PlayerStats,
};
use rusqlite::{Connection, Result};
use serde_json::Value;
use std::collections::HashMap;
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
            manual INTEGER NOT NULL DEFAULT 0,
            team_1 TEXT,
            2 TEXT,
        )
        "#,
    )?;

    Ok(conn)
}

pub fn delete_game(conn: &Connection, game_id: u64) -> Result<()> {
    conn.execute("DELETE FROM game WHERE id=?1", [game_id])?;
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

pub fn update_team(conn: &Connection, game_id: u64, team_name: &str, team_no: u8) -> Result<()> {
    let tn = match team_no {
        1 => "team_1",
        2 => "team_2",
        _ => "",
    };
    conn.execute(
        &format!("UPDATE game SET {}=?1 WHERE id=?2", tn).to_string(),
        (team_name, game_id),
    )?;
    Ok(())
}

pub fn insert_game_with_teams(
    conn: &Connection,
    game_id: &str,
    json: &Value,
    team_1: &str,
    team_2: &str,
) -> Result<()> {
    conn.execute(
        r#"
    INSERT INTO game (id, data, team_1, team_2)
    VALUES (?1, ?2, ?3, ?4)
    "#,
        (game_id, json.to_string(), team_1, team_2),
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

pub fn all_games(conn: &Connection) -> rusqlite::Result<Vec<Game>> {
    let mut stmt = conn.prepare(
        r#"
SELECT * FROM game
ORDER BY json_extract(game.data, "$.info.gameEndTimestamp") DESC"#,
    )?;

    let rows = stmt.query_map([], |row| {
        let stats = Game::try_from(row)?;
        Ok(stats)
    });

    let items: Vec<Game> = rows?.collect::<Result<_>>()?;

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

pub fn stats_overall(conn: &Connection) -> Result<schema::OverallStats> {
    let query_str = include_str!("queries/overall.sql");

    conn.query_row(query_str, [], |row| OverallStats::try_from(row))
}

pub fn stats_players(
    conn: &Connection,
    params: [Option<String>; 3],
) -> Result<Vec<schema::PlayerStats>> {
    let query_str = include_str!("queries/players.sql");
    let mut q = conn.prepare(&(query_str.to_owned() + " ORDER BY kda DESC;"))?;

    q.query_map(params, |row| PlayerStats::try_from(row))?
        .collect()
}

pub fn stats_all_champions(conn: &Connection) -> Result<Vec<schema::ChampionStats>> {
    let query_str = include_str!("queries/all_champs.sql");
    let mut q = conn.prepare(&(query_str.to_owned()))?;

    q.query_map([], |row| ChampionStats::try_from(row))?
        .collect()
}

pub fn stats_champion_history(conn: &Connection) -> Result<HashMap<String, Vec<ChampionHistory>>> {
    let query_str = include_str!("queries/champion_history.sql");
    let mut q = conn.prepare(&(query_str.to_owned()))?;

    q.query_map([], |row| {
        let stats = ChampionHistory::try_from(row)?;
        Ok((stats.champion.clone(), stats))
    })?
    .try_fold(
        HashMap::<String, Vec<ChampionHistory>>::new(),
        |mut acc, row| {
            let (player_name, stats) = row?;
            acc.entry(player_name).or_default().push(stats);
            Ok::<_, rusqlite::Error>(acc)
        },
    )
}

pub fn stats_player_overall(
    conn: &Connection,
) -> Result<HashMap<String, schema::PlayerOverallStats>> {
    let query_str = include_str!("queries/players_overall.sql");
    let mut q = conn.prepare(query_str)?;

    q.query_map([], |row| {
        let stats = PlayerOverallStats::try_from(row)?;
        Ok((stats.player_name.clone(), stats))
    })?
    .collect()
}

pub fn stats_player_role(
    conn: &Connection,
) -> Result<HashMap<String, Vec<schema::PlayerRoleStats>>> {
    let query_str = include_str!("queries/players_role.sql");
    let mut q = conn.prepare(query_str)?;

    q.query_map([], |row| {
        let stats = PlayerRoleStats::try_from(row)?;
        Ok((stats.player_name.clone(), stats))
    })?
    .try_fold(
        HashMap::<String, Vec<PlayerRoleStats>>::new(),
        |mut acc, row| {
            let (player_name, stats) = row?;
            acc.entry(player_name).or_default().push(stats);
            Ok::<_, rusqlite::Error>(acc)
        },
    )
}

pub fn stats_player_champion(
    conn: &Connection,
) -> Result<HashMap<String, Vec<schema::PlayerChampionStats>>> {
    let query_str = include_str!("queries/players_champs.sql");
    let mut q = conn.prepare(query_str)?;

    q.query_map([], |row| {
        let stats = PlayerChampionStats::try_from(row)?;
        Ok((stats.player_name.clone(), stats))
    })?
    .try_fold(
        HashMap::<String, Vec<PlayerChampionStats>>::new(),
        |mut acc, row| {
            let (player_name, stats) = row?;
            acc.entry(player_name).or_default().push(stats);
            Ok::<_, rusqlite::Error>(acc)
        },
    )
}
