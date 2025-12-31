use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Terminal;
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        event::EnableMouseCapture,
        execute,
        terminal::{EnterAlternateScreen, enable_raw_mode},
    },
};

use std::{
    error::Error,
    io::{self, Write},
};

mod app;
mod command;
mod db;
mod riot;
mod ui;
use app::{App, CurrentScreen};
use ui::ui;

fn main() {
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

fn new_main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match (key.code, key.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(true),
                _ => {}
            }
            if app.show_search {
                match key.code {
                    KeyCode::Char(c) => {
                        app.search_input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.search_input.pop();
                    }
                    KeyCode::Esc => {
                        app.show_search = false;
                        app.current_screen = app.previous_screen;
                    }
                    _ => {}
                }
                continue;
            }
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('s') => {
                        app.show_search = true;
                        app.current_screen = CurrentScreen::Search;
                    }
                    _ => {}
                },
                CurrentScreen::Search if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {}
                },
                _ => {}
            }
            match key.code {
                KeyCode::Char('q') => {
                    return Ok(true);
                }
                _ => {}
            }
        }
    }
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
