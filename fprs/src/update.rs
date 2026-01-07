use std::{io::ErrorKind, path::PathBuf};

use color_eyre::eyre::{Error, Result, eyre};
use ratatui::widgets::ListState;
use rusqlite::Connection;
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::{App, CurrentScreen, Message},
    command::{APP_NAME, import_manual_matches},
    db,
};

pub fn update(app: &mut App, msg: Message) -> Option<Message> {
    match msg {
        Message::ListEnd => {
            app.game_ids.state.select_last();
        }
        Message::ListStart => {
            app.game_ids.state.select_first();
        }
        Message::ListDown => {
            app.game_ids.state.select_next();
        }
        Message::ListUp => {
            app.game_ids.state.select_previous();
        }
        Message::LoadGameCount => {
            app.game_count = db::game_count(app.db_connection.as_ref()?).unwrap_or(0);
        }
        Message::InputFinished => {
            app.messages.push(app.input.value_and_reset());
            app.show_input = false;
            app.current_screen = app.next_screen.clone();
            let msg = app.post_message.clone();
            app.post_message = None;
            return msg;
        }
        Message::InputCancelled => {
            app.show_input = false;
        }
        Message::InputKey(key) => {
            app.input.handle_event(&key);
        }
        Message::PromptInput => {
            app.input.reset();
            app.show_input = true;
        }
        Message::OpenMain => {
            app.previous_screen = app.current_screen.clone();
            app.current_screen = CurrentScreen::Main;
        }
        Message::OpenImportManual => {
            app.previous_screen = app.current_screen.clone();
            app.next_screen = CurrentScreen::ImportManual;
            app.post_message = Some(Message::DoImportManual);
            return Some(Message::PromptInput);
        }
        Message::DoImportManual => {
            let import_result = do_import_manual(app);
            match import_result {
                Ok(n) => {
                    app.import_message = format!("Imported {} games", n);
                }
                Err(e) => app.import_message = format!("Import failed: {}", e),
            }
        }
        Message::Quit => {
            app.current_screen = CurrentScreen::Quit;
        }
        _ => {}
    }
    None
}

fn do_import_manual(app: &App) -> color_eyre::Result<usize> {
    let db_path = crate::app::db_path(APP_NAME);
    let conn = Connection::open(db_path)?;
    let path = PathBuf::from(app.messages.last().unwrap_or(&String::new()));

    if !path.is_dir() {
        return Err(eyre!("{} is not a directory.", path.display()));
    }

    return Ok(import_manual_matches(&conn, &path)?);
}
