use ratatui::{crossterm::event::Event, widgets::ListState};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{default, path::PathBuf};
use tui_input::Input;

use crate::ui::view::GameList;

pub fn config_dir(app_name: &str) -> PathBuf {
    let mut dir = dirs::config_dir().expect("Could not determine config directory");

    dir.push(app_name);
    dir
}

pub fn db_path(app_name: &str) -> PathBuf {
    let mut path = config_dir(app_name);
    path.push("app.db");
    path
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    api_key: String,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CurrentScreen {
    Start,
    Main,
    ImportManual,
    Search,
    Quit,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub previous_screen: CurrentScreen,
    pub next_screen: CurrentScreen,
    // Input controls
    pub show_input: bool,
    pub input_title: String,
    pub input: Input,
    pub messages: Vec<String>,
    pub post_message: Option<Message>,
    // Config
    pub config: Config,
    // Import stuff
    pub import_message: String,
    pub imported_games: Vec<String>,
    // Data stuff
    pub startup: bool,
    pub game_count: i64,
    pub db_connection: Option<Connection>,
    pub test_game: Option<Value>,
    pub game_ids: GameList,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            previous_screen: CurrentScreen::Main,
            next_screen: CurrentScreen::Main,
            show_input: false,
            input_title: String::new(),
            messages: Vec::new(),
            input: Input::new("".to_string()),
            post_message: None,
            config: Config::default(),
            import_message: String::new(),
            imported_games: Vec::new(),
            db_connection: None,
            startup: true,
            game_count: 0,
            test_game: None,
            game_ids: GameList::default(),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Message {
    LoadGameCount,
    StartUp,
    InputKey(Event),
    Quit,
    OpenImportManual,
    OpenMain,
    PromptInput,
    InputFinished,
    InputCancelled,
    DoImportManual,
    ListDown,
    ListUp,
    ListEnd,
    ListStart,
}
