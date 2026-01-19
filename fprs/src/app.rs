use ratatui::{
    crossterm::event::Event,
    widgets::{ListState, TableState},
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, default, path::PathBuf};
use strum::{Display, EnumIter, FromRepr};
use tui_input::Input;

use crate::{
    sql::schema::{ChampionHistory, ChampionStats, OverallStats, PlayerDeepStats, PlayerStats},
    ui::view::GameList,
};

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
    Stats,
    Quit,
}

#[derive(Copy, Clone, PartialEq, Default, Display, FromRepr, EnumIter)]
pub enum StatsTab {
    #[default]
    #[strum(to_string = "Game stats")]
    Game,
    #[strum(to_string = "Player stats")]
    Player,
    #[strum(to_string = "Team stats")]
    Team,
    #[strum(to_string = "Champion stats")]
    Champion,
}

impl StatsTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn title(self) -> String {
        format!("  {self}  ")
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
pub enum AlertType {
    #[default]
    None,
    Warning,
    Error,
    Success,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub previous_screen: CurrentScreen,
    pub next_screen: CurrentScreen,
    // Alert controls
    pub show_alert: bool,
    pub alert_type: AlertType,
    pub alert_message: String,
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
    pub db_games: GameList,
    pub search_games: GameList,
    // Stats stuff
    pub stats_tab: StatsTab,
    pub overall_stats: OverallStats,
    pub players_stats: Vec<PlayerStats>,
    pub players_table_state: TableState,
    pub players_sort_col: u64,
    pub players_sort_dir: i64,
    pub players_role_filter: u64,
    // player depth stats
    pub player_deep_stats: PlayerDeepStats,
    // team stats
    // champ stats
    pub all_champs_stats: Vec<ChampionStats>,
    pub all_champs_state: TableState,
    pub champs_history: HashMap<String, Vec<ChampionHistory>>,
    pub champs_sort_dir: i64,
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
            db_games: GameList::default(),
            search_games: GameList::default(),
            show_alert: false,
            alert_type: AlertType::default(),
            alert_message: String::new(),
            overall_stats: OverallStats::default(),
            players_stats: Vec::new(),
            players_table_state: TableState::default(),
            players_sort_col: 0,
            players_sort_dir: -1,
            players_role_filter: 0,
            player_deep_stats: PlayerDeepStats::default(),
            stats_tab: StatsTab::default(),
            all_champs_stats: Vec::new(),
            all_champs_state: TableState::default(),
            champs_history: HashMap::default(),
            champs_sort_dir: -1,
        }
    }

    pub fn get_config_api_key(&self) -> &str {
        return &self.config.api_key;
    }

    pub fn next_stats_tab(&mut self) {
        self.stats_tab = self.stats_tab.next();
    }
    pub fn previous_stats_tab(&mut self) {
        self.stats_tab = self.stats_tab.previous();
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
    ListLeft,
    ListRight,
    OpenSearch,
    DoSearch,
    AddSearchGame,
    ReloadDatabaseGames,
    OpenAlert,
    CloseAlert,
    OpenStats,
    AddSearchTeam1,
    AddSearchTeam2,
    RemoveGame,
    NextTab,
    PrevTab,
}
