use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use rusqlite::{Connection, Result};
use serde_json::Value;

use crate::app;
use crate::db;
use crate::riot;

const APP_NAME: &str = "fprs";

pub struct AppState {
    pub api_key: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self { api_key: None }
    }
}

pub fn handle_command(cmd: &str, state: &mut AppState) -> Result<()> {
    let db_path = app::db_path(APP_NAME);
    match cmd {
        "import-manual" => {
            let conn = Connection::open(db_path)?;
            let path = prompt_path("Path to JSON directory");

            if !path.is_dir() {
                eprintln!("Not a directory: {}", path.display());
            }

            match import_manual_matches(&conn, &path) {
                Ok(count) => {
                    println!("Imported {count} manual matches");
                }
                Err(e) => {
                    eprintln!("Import failed: {e}");
                }
            }
        }
        "add-game" => {
            let conn = Connection::open(db_path)?;
            fetch(state, &conn);
        }
        "init" => match db::init_db(&db_path) {
            Ok(_) => println!("Initialised database at {}", db_path.display()),
            Err(e) => eprintln!("Init failed: {}", e),
        },
        "help" => {
            println!("Available commands:");
            println!("  init    Initialise app database");
            println!("  add-game    Add a game to the tracker");
            println!("  help");
            println!("  quit | exit");
        }
        "" => {} // ignore empty input
        _ => {
            println!("Unknown command: {}", cmd);
        }
    }
    Ok(())
}

pub fn prompt(label: &str) -> String {
    use std::io::{self, Write};

    print!("{label}: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_path(label: &str) -> PathBuf {
    print!("{label}: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    PathBuf::from(input.trim())
}

fn fetch(state: &mut AppState, conn: &Connection) {
    let api_key = match &state.api_key {
        Some(key) => key.clone(),
        None => {
            let key = prompt("Riot API key");
            state.api_key = Some(key.clone());
            key
        }
    };

    let match_id = prompt("Match ID");
    let game_id = "OC1_".to_owned() + &match_id;

    let region = "sea";

    match riot::fetch_match(&api_key, region, &game_id) {
        Ok(json) => {
            println!("Result: {}", json);
            if let Err(e) = db::insert_game(conn, &match_id, &json) {
                eprintln!("Failed to store game: {e}");
            } else {
                println!("Game retrieved!");
            }
        }
        Err(e) => eprintln!("Fetch failed: {e}"),
    }
}

fn import_manual_matches(conn: &Connection, dir: &Path) -> Result<usize> {
    conn.execute("DELETE FROM game WHERE manual = 1", [])?;

    let mut inserted = 0;

    for entry in fs::read_dir(dir).expect("Failed to read directory") {
        let entry = entry.expect("Invalid directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        if path.file_name().and_then(|s| s.to_str()) == Some("template.json") {
            continue;
        }

        let json = fs::read_to_string(&path).expect("Failed to read JSON file");

        let v: Value = serde_json::from_str(&json).expect("Invalid JSON");

        // extract info.gameId
        let foo = v["info"]["gameId"]
            .as_i64()
            .expect("info.gameId missing or not an integer");

        conn.execute(
            r#"
            INSERT INTO game (id, data, manual)
            VALUES (?1, ?2, 1)
            "#,
            (foo, &json),
        )?;

        inserted += 1;
    }

    Ok(inserted)
}
