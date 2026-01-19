use color_eyre::Result;
use color_eyre::eyre::Context;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use rusqlite::Connection;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::{
    error::Error,
    io::{self, Write},
};

mod app;
mod command;
mod riot;
mod sql;
mod ui;
mod update;
use app::{App, CurrentScreen};
use command::APP_NAME;
use sql::repo;
use ui::view;

use crate::app::{AlertType, Config, Message, config_dir};
use crate::ui::GameList;

fn old_main() {
    let mut state = command::AppState::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let input = input.trim();

        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }

        // Dispatch commands
        let _ = command::handle_command(input, &mut state);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    init_config()?;
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;

    let mut app = App::new();
    app.config = read_config()?;
    app.db_connection = Some(init_database()?);
    // app.game_count = repo::game_count(app.db_connection.as_ref().unwrap()).unwrap_or(0);
    app.test_game = Some(repo::game_by_id(app.db_connection.as_ref().unwrap(), 1)?);
    match repo::all_games(app.db_connection.as_ref().unwrap()) {
        Ok(games) => {
            app.db_games = GameList::from_iter(games);
        }
        Err(e) => {
            app.alert_type = AlertType::Error;
            app.alert_message = format!("{e}");
            app.show_alert = true;
        }
    }
    app.game_count = app.db_games.len() as i64;

    while app.current_screen != CurrentScreen::Quit {
        terminal.draw(|f| view::view(f, &mut app))?;

        let mut current_msg = handle_event(&app)?;

        while current_msg.is_some() {
            current_msg = update::update(&mut app, current_msg.unwrap());
        }
    }

    tui::restore_terminal()?;
    Ok(())
}

fn init_config() -> io::Result<PathBuf> {
    // Create toml if not exist
    let mut config_path = app::config_dir(APP_NAME);
    config_path.push("config.toml");

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(
            &config_path,
            r#"api_key = ""
"#,
        )?;
    }

    Ok(config_path)
}

fn init_database() -> rusqlite::Result<Connection> {
    let db_path = app::db_path(APP_NAME);
    return repo::init_db(&db_path);
}

fn read_config() -> Result<Config> {
    let mut path = config_dir(APP_NAME);
    path.push("config.toml");
    init_config()?;

    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    let cfg: Config = toml::from_str(&text).context("invalid config.toml")?;

    Ok(cfg)
}

fn write_config(cfg: &Config) -> Result<()> {
    let mut path = config_dir(APP_NAME);
    path.push("config.toml");

    let toml = toml::to_string_pretty(cfg).context("failed to serialise config")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, toml).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn handle_event(app: &App) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(app, key));
            }
        }
    }
    Ok(None)
}

fn handle_key(app: &App, key: event::KeyEvent) -> Option<Message> {
    // Above ALL else, if CTRL-c is pressed, quit the program
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            return Some(Message::Quit);
        }
        _ => {}
    }
    // If the result floater is displaying, absorb all keys
    if app.show_alert {
        return Some(Message::CloseAlert);
    }
    // Then if the input floater is displaying, absorb all keys
    if app.show_input {
        match key.code {
            KeyCode::Esc => return Some(Message::InputCancelled),
            KeyCode::Enter => return Some(Message::InputFinished),
            _ => {
                return Some(Message::InputKey(Event::Key(key)));
            }
        }
    }
    // Global key strokes
    match key.code {
        KeyCode::Char('q') => {
            return Some(Message::Quit);
        }
        KeyCode::Char('j') | KeyCode::Down => return Some(Message::ListDown),
        KeyCode::Char('k') | KeyCode::Up => return Some(Message::ListUp),
        KeyCode::Char('J') | KeyCode::End => return Some(Message::ListEnd),
        KeyCode::Char('K') | KeyCode::Home => return Some(Message::ListStart),
        KeyCode::Char('h') | KeyCode::Left => return Some(Message::ListLeft),
        KeyCode::Char('l') | KeyCode::Right => return Some(Message::ListRight),
        _ => {}
    }
    // Screen-specific strokes
    match app.current_screen {
        CurrentScreen::Main | CurrentScreen::Start => match key.code {
            KeyCode::Delete => Some(Message::RemoveGame),
            KeyCode::Char('i') => Some(Message::OpenImportManual),
            KeyCode::Char('f') => Some(Message::OpenSearch),
            KeyCode::Char('s') => Some(Message::OpenStats),
            _ => None,
        },
        CurrentScreen::ImportManual => match key.code {
            KeyCode::Esc => Some(Message::OpenMain),
            _ => None,
        },
        CurrentScreen::Stats => match key.code {
            KeyCode::Esc => Some(Message::OpenMain),
            KeyCode::Char('[') => Some(Message::PrevTab),
            KeyCode::Char(']') => Some(Message::NextTab),
            _ => Some(Message::InputKey(Event::Key(key))),
        },
        CurrentScreen::Search => match key.code {
            KeyCode::Enter => Some(Message::AddSearchGame),
            KeyCode::Esc => Some(Message::OpenMain),
            _ => None,
        },
        _ => None,
    }
}

mod tui {
    use ratatui::{
        Terminal,
        backend::{Backend, CrosstermBackend},
        crossterm::{
            ExecutableCommand,
            terminal::{
                EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
            },
        },
    };
    use std::{io::stdout, panic};

    pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> color_eyre::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}
