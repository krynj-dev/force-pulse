use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::path::PathBuf;

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

#[derive(Copy, Clone)]
pub enum CurrentScreen {
    Main,
    Search,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub previous_screen: CurrentScreen,
    pub show_search: bool,
    pub search_input: String,
    pub search_buf: String,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            previous_screen: CurrentScreen::Main,
            show_search: false,
            search_input: String::new(),
            search_buf: String::new(),
        }
    }

    // pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
    //     while !self.exit {
    //         terminal.draw(|frame| self.draw(frame))?;
    //         self.handle_events()?;
    //     }
    //     Ok(())
    // }
    //
    // fn draw(&self, frame: &mut Frame) {
    //     frame.render_widget(self, frame.area());
    // }
    //
    // fn handle_events(&mut self) -> io::Result<()> {
    //     match event::read()? {
    //         // it's important to check that the event is a key press event as
    //         // crossterm also emits key release and repeat events on Windows.
    //         Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
    //             self.handle_key_event(key_event)
    //         }
    //         _ => {}
    //     };
    //     Ok(())
    // }
    //
    // fn handle_key_event(&mut self, key_event: KeyEvent) {
    //     match key_event.code {
    //         KeyCode::Char('q') => self.exit(),
    //         KeyCode::Left => self.decrement_counter(),
    //         KeyCode::Right => self.increment_counter(),
    //         _ => {}
    //     }
    // }
    //
    // fn exit(&mut self) {
    //     self.exit = true;
    // }
    //
    // fn increment_counter(&mut self) {
    //     self.counter += 1;
    // }
    //
    // fn decrement_counter(&mut self) {
    //     self.counter -= 1;
    // }
}

// impl Widget for &App {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let line = Line::from(" Counter App Tutorial ".bold());
//         let title = line;
//         let instructions = Line::from(vec![
//             " Decrement ".into(),
//             "<Left>".blue().bold(),
//             " Increment ".into(),
//             "<Right>".blue().bold(),
//             " Quit ".into(),
//             "<Q> ".blue().bold(),
//         ]);
//         let block = Block::bordered()
//             .title(title.centered())
//             .title_bottom(instructions.centered())
//             .border_set(border::THICK);
//
//         let counter_text = Text::from(vec![Line::from(vec![
//             "Value: ".into(),
//             self.counter.to_string().yellow(),
//         ])]);
//
//         Paragraph::new(counter_text)
//             .centered()
//             .block(block)
//             .render(area, buf);
//     }
// }
