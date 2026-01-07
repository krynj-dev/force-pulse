use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::tailwind::SLATE},
    symbols::line::{self, Set},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
    },
};
use serde_json::Value;

use crate::{
    app::{App, CurrentScreen},
    ui::draw_scoreboard,
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

pub fn view(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        format!("LoL Stat Tracker | {} games tracked.", app.game_count),
        Style::default().fg(Color::Green),
    ))
    .block(title_block.clone());

    frame.render_widget(title, chunks[0]);

    match app.current_screen {
        CurrentScreen::Main | CurrentScreen::Start => render_main(frame, chunks[1], app),
        CurrentScreen::ImportManual => render_import(frame, chunks[1], app),
        CurrentScreen::Search => render_search(frame, chunks[1], app),
        CurrentScreen::Quit => render_quit(frame, chunks[1], app),
    }

    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Main | CurrentScreen::Start => {
                Span::styled("Normal Mode", Style::default().fg(Color::Green))
            }
            CurrentScreen::ImportManual => {
                Span::styled("Import manual data", Style::default().fg(Color::Cyan))
            }
            CurrentScreen::Search => Span::styled("Search", Style::default().fg(Color::LightRed)),
            CurrentScreen::Quit => Span::styled("Search", Style::default().fg(Color::Red)),
        }
        .to_owned(),
        // A white divider bar to separate the two sections
        Span::styled(" | ", Style::default().fg(Color::White)),
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Main | CurrentScreen::Quit | CurrentScreen::Start => Span::styled(
                "(q) to quit, (i) to import manual data",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::ImportManual => match app.show_input {
                true => Span::styled(
                    "Enter the path to the manual data",
                    Style::default().fg(Color::Red),
                ),
                false => Span::styled("<Esc> to return to home", Style::default().fg(Color::Red)),
            },
            CurrentScreen::Search => Span::styled(
                "(ESC) to cancel/(Tab) to switch boxes/enter to complete",
                Style::default().fg(Color::Red),
            ),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);

    if app.show_input {
        let popup = centered_rect(50, 5, frame.size());

        render_input(app, frame, popup);
    }
}

fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    // keep 2 for borders and 1 for cursor
    let width = area.width.max(3) - 3;
    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .scroll((0, scroll as u16))
        .block(Block::bordered().title("Input"));
    frame.render_widget(input, area);

    // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
    // end of the input text and one line down from the border to the input line
    let x = app.input.visual_cursor().max(scroll) - scroll + 1;
    frame.set_cursor_position((area.x + x as u16, area.y + 1))
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

#[derive(Default)]
pub struct GameList {
    items: Vec<(String, Value)>,
    pub state: ListState,
}

impl FromIterator<(String, Value)> for GameList {
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        let items = iter.into_iter().collect();
        let state = ListState::default();
        Self { items, state }
    }
}

impl GameList {
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

fn render_main(frame: &mut Frame, area: Rect, app: &mut App) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let inner_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(15), Constraint::Min(1)])
        .split(area);

    let items: Vec<ListItem> = app
        .game_ids
        .items
        .iter()
        .enumerate()
        .map(|(i, (s, _))| ListItem::from(s.as_str()).bg(alternate_colors(i)))
        .collect();

    let game_list = List::new(items)
        .block(title_block.clone().title("Games"))
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

    StatefulWidget::render(
        game_list,
        inner_chunks[0],
        frame.buffer_mut(),
        &mut app.game_ids.state,
    );

    draw_scoreboard(
        frame.buffer_mut(),
        inner_chunks[1],
        match app.game_ids.state.selected() {
            Some(i) => &(app.game_ids.items[i].1),
            None => &Value::Null,
        },
    );
}

fn render_import(frame: &mut Frame, area: Rect, app: &mut App) {
    let central_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let content = Paragraph::new(Text::styled(app.import_message.clone(), Style::default()))
        .block(central_block);

    frame.render_widget(content, area);
}

fn render_search(frame: &mut Frame, area: Rect, app: &mut App) {
    let central_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let content = Paragraph::new(Text::styled(
        "I'm searching now",
        Style::default().fg(Color::Yellow),
    ))
    .block(central_block);

    frame.render_widget(content, area);
}

fn render_quit(frame: &mut Frame, area: Rect, app: &mut App) {
    let central_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let content = Paragraph::new(Text::styled(
        "Quitting...",
        Style::default().fg(Color::Yellow),
    ))
    .block(central_block);

    frame.render_widget(content, area);
}
