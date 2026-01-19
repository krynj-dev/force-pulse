use color_eyre::owo_colors::OwoColorize;
use pad::PadStr;
use reqwest::header;
use std::{cmp::Ordering, collections::HashMap, fmt::format};
use unicode_width::UnicodeWidthChar;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{
        Color, Modifier, Style,
        palette::tailwind::{self, SLATE},
    },
    symbols::{
        border::{self, Set},
        line,
    },
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, StatefulWidget, Table,
        Widget,
    },
};
use serde_json::Value;
use unicode_width::UnicodeWidthStr;

use crate::{
    app::App,
    sql::schema::{
        ChampionHistory, Game, PlayerChampionStats, PlayerOverallStats, PlayerRoleStats,
    },
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const ROLES: [&'static str; 5] = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: color.c400,
            selected_column_style_fg: color.c400,
            selected_cell_style_fg: color.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

fn role_order(role: &str) -> usize {
    match role {
        "TOP" => 0,
        "JUNGLE" => 1,
        "MIDDLE" => 2,
        "BOTTOM" => 3,
        "UTILITY" => 4,
        _ => 5,
    }
}

pub const DASHED: Set = Set {
    top_left: line::NORMAL.top_left,
    top_right: line::NORMAL.top_right,
    bottom_left: line::NORMAL.bottom_left,
    bottom_right: line::NORMAL.bottom_right,
    vertical_left: "╎",
    vertical_right: "╎",
    horizontal_top: "╌",
    horizontal_bottom: "╌",
};

fn team_order(team: &u64) -> usize {
    match team {
        100 => 0,
        200 => 1,
        _ => 2,
    }
}

pub fn draw_scoreboard(buf: &mut Buffer, area: Rect, game: &Game) {
    let game_data = game.data.clone();
    // Parent block
    let game_duration = game_data
        .get("info")
        .and_then(|i| i.get("gameDuration"))
        .and_then(|d| d.as_u64())
        .unwrap_or_else(|| 0);

    let game_end_timestamp = Local
        .timestamp_opt(
            game_data
                .get("info")
                .and_then(|i| i.get("gameEndTimestamp"))
                .and_then(|d| d.as_i64())
                .unwrap_or_else(|| 0)
                / 1_000,
            0,
        )
        .unwrap();

    let block = Block::default()
        .title(format!(
            " {} | {:02}:{:02} | {} ",
            game_data
                .get("info")
                .and_then(|info| info.get("gameId"))
                .and_then(|gid| gid.as_u64())
                .map(|gid| gid.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            game_duration / 60,
            game_duration % 60,
            game_end_timestamp.format("%d/%m/%Y").to_string()
        ))
        .borders(Borders::ALL)
        .style(Style::default());

    let block_inner = block.inner(area);

    block.render(area, buf);
    // Give it a layout
    let inner_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Max(6),
            Constraint::Max(1),
            Constraint::Max(6),
            Constraint::Min(1),
        ])
        .split(block_inner);
    // Define inner block style
    let team_block = Block::default()
        .borders(Borders::TOP)
        .border_set(DASHED)
        .style(Style::default());

    let col_widths = [18, 12, 10, 7, 7];

    let mut lines: Vec<Line> = Vec::new();

    let mut participants = game_data
        .get("info")
        .and_then(|info| info.get("participants"))
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or(Vec::new());

    participants.sort_by_key(|p| {
        let role = p.get("teamPosition").and_then(|v| v.as_str()).unwrap_or("");
        let team = p
            .get("teamId")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| 0);

        (team_order(&team), role_order(role))
    });

    let mut stats: HashMap<u64, (u64, u64, u64, u64, u64)> = HashMap::new();
    let mut blue_win: bool = false;

    for i in 0..participants.len() / 2 {
        let p = participants.get(i).unwrap_or_else(|| &Value::Null);
        let team_id = p.get("teamId").and_then(|t| t.as_u64()).unwrap_or(0);
        let kills = p.get("kills").and_then(|n| n.as_u64()).unwrap_or(0);
        let deaths = p.get("deaths").and_then(|n| n.as_u64()).unwrap_or(0);
        let assists = p.get("assists").and_then(|n| n.as_u64()).unwrap_or(0);
        let damage = p
            .get("totalDamageDealtToChampions")
            .and_then(|n| n.as_u64())
            .unwrap_or(0);
        let gold = p.get("goldEarned").and_then(|n| n.as_u64()).unwrap_or(0);

        blue_win = p.get("win").and_then(|w| w.as_bool()).unwrap_or(false);

        stats
            .entry(team_id)
            .and_modify(|e| {
                e.0 += kills;
                e.1 += deaths;
                e.2 += assists;
                e.3 += damage;
                e.4 += gold;
            })
            .or_insert((kills, deaths, assists, damage, gold));
        lines.push(map_participant_line(p, &col_widths));
    }
    let (k, d, a, _, g) = stats.get(&100).cloned().unwrap_or((0, 0, 0, 0, 0));
    Paragraph::new(Text::from(lines.clone()))
        .block(team_block.clone().title(format!(
            " {} | 󰞇 {:>2}/{:>2}/{:>2} |  {:.1}k | {} ",
            game.team_1.to_string(),
            k,
            d,
            a,
            g.to_owned() as f64 / 1000f64,
            if blue_win { "Victory" } else { "Loss" },
        )))
        .render(inner_chunks[0], buf);
    lines.clear();

    for i in participants.len() / 2..participants.len() {
        let p = participants.get(i).unwrap_or_else(|| &Value::Null);
        let team_id = p.get("teamId").and_then(|t| t.as_u64()).unwrap_or(0);
        let kills = p.get("kills").and_then(|n| n.as_u64()).unwrap_or(0);
        let deaths = p.get("deaths").and_then(|n| n.as_u64()).unwrap_or(0);
        let assists = p.get("assists").and_then(|n| n.as_u64()).unwrap_or(0);
        let damage = p
            .get("totalDamageDealtToChampions")
            .and_then(|n| n.as_u64())
            .unwrap_or(0);
        let gold = p.get("goldEarned").and_then(|n| n.as_u64()).unwrap_or(0);
        stats
            .entry(team_id)
            .and_modify(|e| {
                e.0 += kills;
                e.1 += deaths;
                e.2 += assists;
                e.3 += damage;
                e.4 += gold;
            })
            .or_insert((kills, deaths, assists, damage, gold));
        lines.push(map_participant_line(p, &col_widths));
    }

    let (k, d, a, _, g) = stats.get(&200).cloned().unwrap_or((0, 0, 0, 0, 0));
    Paragraph::new(Text::from(lines.clone()))
        .block(team_block.clone().title(format!(
            " {} | 󰞇 {:>2}/{:>2}/{:>2} |  {:.1}k | {} ",
            game.team_2.to_string(),
            k,
            d,
            a,
            g.to_owned() as f64 / 1000f64,
            if !blue_win { "Victory" } else { "Loss" },
        )))
        .render(inner_chunks[2], buf);
    lines.clear();
}

fn map_participant_line<'a>(p: &Value, col_widths: &[usize; 5]) -> Line<'a> {
    let player_name = p
        .get("riotIdGameName")
        .and_then(|n| n.as_str())
        .unwrap_or_else(|| "Unknown");
    let trunc_name = truncate_with_ellipsis_width(player_name, col_widths[0]);
    let champion_name = p
        .get("championName")
        .and_then(|n| n.as_str())
        .unwrap_or_else(|| "Unknown");
    let kills = p.get("kills").and_then(|n| n.as_u64()).unwrap_or_else(|| 0);
    let deaths = p
        .get("deaths")
        .and_then(|n| n.as_u64())
        .unwrap_or_else(|| 0);
    let assists = p
        .get("assists")
        .and_then(|n| n.as_u64())
        .unwrap_or_else(|| 0);
    let damage = p
        .get("totalDamageDealtToChampions")
        .and_then(|n| n.as_u64())
        .unwrap_or_else(|| 0);
    let gold = p
        .get("goldEarned")
        .and_then(|n| n.as_u64())
        .unwrap_or_else(|| 0);
    Line::from(vec![
        Span::styled(
            format!(
                " {:>} ",
                trunc_name.pad(col_widths[0], ' ', pad::Alignment::Right, false)
            ),
            Style::default(),
        ),
        Span::styled(
            format!(" {:>width$} ", champion_name, width = col_widths[1]),
            Style::default(),
        ),
        Span::styled(
            format!(
                " {:<width$} ",
                format!("󰞇 {:>2}/{:>2}/{:>2}", kills, deaths, assists),
                width = col_widths[2]
            ),
            Style::default(),
        ),
        Span::styled(
            format!(
                " {:>width$} ",
                format!("󰓥 {:2.1}k", damage as f64 / 1000f64),
                width = col_widths[3]
            ),
            Style::default(),
        ),
        Span::styled(
            format!(
                " {:>width$} ",
                format!(" {:2.1}k", gold as f64 / 1000f64),
                width = col_widths[4]
            ),
            Style::default(),
        ),
    ])
}

fn truncate_with_ellipsis_width(s: &str, max_width: usize) -> String {
    let mut w = 0;
    let mut truncated = String::new();

    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if w + cw >= max_width.saturating_sub(1) {
            truncated.push('…');
            break;
        }
        truncated.push(c);
        w += cw;
    }
    truncated
}

pub fn draw_overall(buf: &mut Buffer, area: Rect, app: &App) {
    let block = Block::default()
        .title("Global stats")
        .borders(Borders::ALL)
        .style(Style::default());

    let header = [
        "Games",
        "Blue Wins",
        "Red Wins",
        "Shortest Game",
        "Average Game",
        "Longest Game",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(Style::default())
    .height(1);
    let stats = &app.overall_stats;
    let rows = [Row::from_iter(vec![
        Cell::from(Text::from(stats.games.to_string())),
        Cell::from(Text::from(stats.blue_wins.to_string())),
        Cell::from(Text::from(stats.red_wins.to_string())),
        Cell::from(Text::from(format!(
            "{:02}:{:02}",
            stats.game_length_min / 60,
            stats.game_length_min % 60
        ))),
        Cell::from(Text::from(format!(
            "{:02}:{:02}",
            stats.game_length_avg / 60,
            stats.game_length_avg % 60
        ))),
        Cell::from(Text::from(format!(
            "{:02}:{:02}",
            stats.game_length_max / 60,
            stats.game_length_max % 60
        ))),
    ])];

    let t = Table::new(
        rows,
        [
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
        ],
    )
    .header(header)
    .block(block);

    Widget::render(t, area, buf);
}

pub fn draw_players(buf: &mut Buffer, area: Rect, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(23), Constraint::Min(1)])
        .split(area);
    let sublay = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Min(1)])
        .split(layout[1]);
    let colours = TableColors::new(&tailwind::BLUE);

    let block = Block::default()
        .title("Players stats")
        .borders(Borders::ALL)
        .style(Style::default());

    let header = [
        "#", "Riot ID", // "Tagline",
        "Team", "Role", "Games", "Kills", "Deaths", "Assists", "KDA", "GPM", "CSPM", "CSD@10",
        "Kill %", "Death %", "DPM", "VSPM",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .height(3)
    .style(Style::default().fg(colours.header_fg).bg(colours.header_bg))
    .height(1);
    let mut stats = app.players_stats.clone();
    let sel_col = app
        .players_table_state
        .selected_column()
        .unwrap_or_else(|| 1);

    stats.retain(|s| match app.players_role_filter {
        0 => true,
        i if i > 6 => true,
        i => s.role == ROLES[(i - 1) as usize],
    });

    stats.sort_by(|a, b| match sel_col {
        1 => a.riot_id.to_string().cmp(&b.riot_id.to_string()),
        2 => a.team_name.to_string().cmp(&b.team_name.to_string()),
        3 => a.role.to_string().cmp(&b.role.to_string()),
        4 => a.games.cmp(&b.games),
        5 => a.kills.cmp(&b.kills),
        6 => a.deaths.cmp(&b.deaths),
        7 => a.assists.cmp(&b.assists),
        8 => match a.kda.partial_cmp(&b.kda) {
            Some(Ordering::Less) => Ordering::Less,
            Some(Ordering::Greater) => Ordering::Greater,
            _ => Ordering::Equal,
        },
        9 => a.gpm.cmp(&b.gpm),
        10 => match a.cspm.partial_cmp(&b.cspm) {
            Some(Ordering::Less) => Ordering::Less,
            Some(Ordering::Greater) => Ordering::Greater,
            _ => Ordering::Equal,
        },
        11 => match a.cd10.partial_cmp(&b.cd10) {
            Some(Ordering::Less) => Ordering::Less,
            Some(Ordering::Greater) => Ordering::Greater,
            _ => Ordering::Equal,
        },
        12 => a.kill_participation.cmp(&b.kill_participation),
        13 => a.death_participation.cmp(&b.death_participation),
        14 => a.dpm.cmp(&b.dpm),
        15 => match a.vpm.partial_cmp(&b.vpm) {
            Some(Ordering::Less) => Ordering::Less,
            Some(Ordering::Greater) => Ordering::Greater,
            _ => Ordering::Equal,
        },
        _ => Ordering::Equal,
    });

    match app.players_sort_dir {
        -1 => {
            stats.reverse();
        }
        _ => {}
    }
    let rows = stats.iter().enumerate().map(|(i, stat)| {
        let colour = match i % 2 {
            0 => colours.normal_row_color,
            _ => colours.alt_row_color,
        };
        Row::from_iter(vec![
            Cell::from(Text::from((i + 1).to_string())),
            Cell::from(Text::from(stat.riot_id.clone())),
            // Cell::from(Text::from(stat.tag_line.clone())),
            Cell::from(Text::from(stat.team_name.clone())),
            Cell::from(Text::from(stat.role.clone())),
            Cell::from(Text::from(stat.games.to_string())),
            Cell::from(Text::from(stat.kills.to_string())),
            Cell::from(Text::from(stat.deaths.to_string())),
            Cell::from(Text::from(stat.assists.to_string())),
            Cell::from(Text::from(stat.kda.to_string())),
            Cell::from(Text::from(stat.gpm.to_string())),
            Cell::from(Text::from(stat.cspm.to_string())),
            Cell::from(Text::from(match stat.cd10 {
                Some(d) => d.to_string(),
                None => "-".to_string(),
            })),
            Cell::from(Text::from(stat.kill_participation.to_string())),
            Cell::from(Text::from(stat.death_participation.to_string())),
            Cell::from(Text::from(match stat.dpm {
                Some(d) => d.to_string(),
                None => "-".to_string(),
            })),
            Cell::from(Text::from(match stat.vpm {
                Some(d) => format!("{:.2}", d),
                None => "-".to_string(),
            })),
        ])
        .style(Style::new().fg(colours.row_fg).bg(colour))
    });

    let t = Table::new(
        rows,
        [
            Constraint::Max(4),
            // Constraint::Min(1),
            Constraint::Max(15),
            Constraint::Max(10),
            Constraint::Max(7),
            Constraint::Max(5),
            Constraint::Max(7),
            Constraint::Max(7),
            Constraint::Max(7),
            Constraint::Max(5),
            Constraint::Max(4),
            Constraint::Max(4),
            Constraint::Max(7),
            Constraint::Max(7),
            Constraint::Max(7),
            Constraint::Max(5),
            Constraint::Min(1),
        ],
    )
    .header(header)
    .row_highlight_style(SELECTED_STYLE)
    .column_highlight_style(SELECTED_STYLE)
    .highlight_spacing(HighlightSpacing::Always)
    .block(block);

    StatefulWidget::render(t, layout[0], buf, &mut app.players_table_state);

    match app.players_table_state.selected() {
        Some(i) => {
            draw_players_deep(
                buf,
                sublay[0],
                app,
                stats.get(i).unwrap().riot_id.clone(),
                stats.get(i).unwrap().tag_line.clone(),
            );
        }
        None => Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .render(layout[1], buf),
    }
}

pub fn draw_players_deep(buf: &mut Buffer, area: Rect, app: &mut App, plr: String, tag: String) {
    let colours = TableColors::new(&tailwind::BLUE);
    let block = Block::default()
        .title(format!("{}#{}", plr, tag))
        .borders(Borders::ALL)
        .style(Style::default());

    let header = [
        "", "Games", "Wins", "Losses", "Win %", "KPG", "DPG", "APG", "KDA", "GPM", "DPM", "CSD@10",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(Style::default().fg(colours.header_fg).bg(colours.header_bg))
    .height(1);
    let overall_stats = &app.player_deep_stats.overall_stats.get(&plr).unwrap();
    let role_stats = &app.player_deep_stats.role_stats.get(&plr).unwrap();
    let champ_stats = &app.player_deep_stats.champion_stats.get(&plr).unwrap();
    let mut rows: Vec<Row> = Vec::new();
    rows.extend(get_overall_rows(&colours, overall_stats));
    rows.extend(get_role_rows(&colours, role_stats));
    rows.extend(get_champs_rows(&colours, champ_stats));

    let t = Table::new(
        rows,
        [
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
        ],
    )
    .header(header)
    .block(block);

    Widget::render(t, area, buf);
}

fn get_overall_rows(
    colours: &TableColors,
    overall_stats: &PlayerOverallStats,
) -> Vec<Row<'static>> {
    vec![
        Row::from_iter(vec![
            Cell::from(Text::from("ALL".to_string())),
            Cell::from(Text::from(overall_stats.games.to_string())),
            Cell::from(Text::from(overall_stats.wins.to_string())),
            Cell::from(Text::from(overall_stats.losses.to_string())),
            Cell::from(Text::from(overall_stats.win_percent.to_string())),
            Cell::from(Text::from(overall_stats.kills_per_game.to_string())),
            Cell::from(Text::from(overall_stats.deaths_per_game.to_string())),
            Cell::from(Text::from(overall_stats.assists_per_game.to_string())),
            Cell::from(Text::from(overall_stats.kda.to_string())),
            Cell::from(Text::from(overall_stats.gpm.to_string())),
            Cell::from(Text::from(match overall_stats.dpm {
                Some(i) => i.to_string(),
                None => "-".to_string(),
            })),
            Cell::from(Text::from(match overall_stats.cd10 {
                Some(i) => i.to_string(),
                None => "-".to_string(),
            })),
        ])
        .style(Style::new().fg(colours.row_fg).bg(colours.normal_row_color)),
        Row::from_iter(vec![
            Cell::new(Text::from("".to_string())),
            Cell::new(Text::from("".to_string())),
            Cell::new(Text::from("".to_string())),
            Cell::new(Text::from("".to_string())),
            Cell::new(Text::from("".to_string())),
            Cell::from(Text::from(format!(
                "{}/{}",
                overall_stats.kpgn, overall_stats.total
            ))),
            Cell::from(Text::from(format!(
                "{}/{}",
                overall_stats.dpgn, overall_stats.total
            ))),
            Cell::from(Text::from(format!(
                "{}/{}",
                overall_stats.apgn, overall_stats.total
            ))),
            Cell::from(Text::from(format!(
                "{}/{}",
                overall_stats.kdan, overall_stats.total
            ))),
            Cell::from(Text::from(format!(
                "{}/{}",
                overall_stats.gpmn, overall_stats.total
            ))),
            Cell::from(Text::from(format!(
                "{}/{}",
                match overall_stats.dpmn {
                    Some(i) => i.to_string(),
                    None => "-".to_string(),
                },
                overall_stats.total
            ))),
            Cell::from(Text::from(format!(
                "{}/{}",
                match overall_stats.cd10n {
                    Some(i) => i.to_string(),
                    None => "-".to_string(),
                },
                overall_stats.total
            ))),
        ]),
    ]
}

fn get_role_rows(colours: &TableColors, role_stats: &Vec<PlayerRoleStats>) -> Vec<Row<'static>> {
    role_stats
        .iter()
        .flat_map(|v| {
            vec![
                Row::from_iter(vec![
                    Cell::from(Text::from(v.role.to_string())),
                    Cell::from(Text::from(v.games.to_string())),
                    Cell::from(Text::from(v.wins.to_string())),
                    Cell::from(Text::from(v.losses.to_string())),
                    Cell::from(Text::from(v.win_percent.to_string())),
                    Cell::from(Text::from(v.kills_per_game.to_string())),
                    Cell::from(Text::from(v.deaths_per_game.to_string())),
                    Cell::from(Text::from(v.assists_per_game.to_string())),
                    Cell::from(Text::from(v.kda.to_string())),
                    Cell::from(Text::from(v.gpm.to_string())),
                    Cell::from(Text::from(match v.dpm {
                        Some(i) => i.to_string(),
                        None => "-".to_string(),
                    })),
                    Cell::from(Text::from(match v.cd10 {
                        Some(i) => i.to_string(),
                        None => "-".to_string(),
                    })),
                ])
                .style(Style::new().fg(colours.row_fg).bg(colours.normal_row_color)),
                Row::from_iter(vec![
                    Cell::new(Text::from("".to_string())),
                    Cell::new(Text::from("".to_string())),
                    Cell::new(Text::from("".to_string())),
                    Cell::new(Text::from("".to_string())),
                    Cell::new(Text::from("".to_string())),
                    Cell::from(Text::from(format!("{}/{}", v.kpgn, v.role_total))),
                    Cell::from(Text::from(format!("{}/{}", v.dpgn, v.role_total))),
                    Cell::from(Text::from(format!("{}/{}", v.apgn, v.role_total))),
                    Cell::from(Text::from(format!("{}/{}", v.kdan, v.role_total))),
                    Cell::from(Text::from(format!("{}/{}", v.gpmn, v.role_total))),
                    Cell::from(Text::from(format!(
                        "{}/{}",
                        match v.dpmn {
                            Some(i) => i.to_string(),
                            None => "-".to_string(),
                        },
                        v.role_total
                    ))),
                    Cell::from(Text::from(format!(
                        "{}/{}",
                        match v.cd10n {
                            Some(i) => i.to_string(),
                            None => "-".to_string(),
                        },
                        v.role_total
                    ))),
                ])
                .style(Style::new().fg(colours.row_fg).bg(colours.alt_row_color)),
            ]
        })
        .collect()
}

fn get_champs_rows(
    colours: &TableColors,
    champ_stats: &Vec<PlayerChampionStats>,
) -> Vec<Row<'static>> {
    champ_stats
        .iter()
        .flat_map(|v| {
            vec![
                Row::from_iter(vec![
                    Cell::from(Text::from(v.champion_name.to_string())),
                    Cell::from(Text::from(v.games.to_string())),
                    Cell::from(Text::from(v.wins.to_string())),
                    Cell::from(Text::from(v.losses.to_string())),
                    Cell::from(Text::from(v.win_percent.to_string())),
                    Cell::from(Text::from(v.kills_per_game.to_string())),
                    Cell::from(Text::from(v.deaths_per_game.to_string())),
                    Cell::from(Text::from(v.assists_per_game.to_string())),
                    Cell::from(Text::from(v.kda.to_string())),
                    Cell::from(Text::from(v.gpm.to_string())),
                    Cell::from(Text::from(match v.dpm {
                        Some(i) => i.to_string(),
                        None => "-".to_string(),
                    })),
                    Cell::from(Text::from(match v.cd10 {
                        Some(i) => i.to_string(),
                        None => "-".to_string(),
                    })),
                ])
                .style(Style::new().fg(colours.row_fg).bg(colours.normal_row_color)),
            ]
        })
        .collect()
}

pub fn draw_champions(buf: &mut Buffer, area: Rect, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Min(1)])
        .split(area);
    let colours = TableColors::new(&tailwind::ORANGE);

    let block = Block::default()
        .title("Champion stats")
        .borders(Borders::ALL)
        .style(Style::default());

    let header = [
        "#", "Champion", "Games", "Pick %", "Players", "Wins", "Losses", "Wins %", "Kills",
        "Deaths", "Assists", "KDA", "CS", "CS/M", "VS", "VS/M", "Gold", "G/M", "Damage", "DMG/M",
        "KPAR", "KS", "GS", "Roles",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(Style::default().fg(colours.header_fg).bg(colours.header_bg))
    .height(1);

    let mut stats = app.all_champs_stats.clone();
    let sel_col = app.all_champs_state.selected_column().unwrap_or_else(|| 1);

    match sel_col {
        1 => stats.sort_by_key(|s| s.champion.clone()),
        2 => stats.sort_by_key(|s| s.games),
        4 => stats.sort_by_key(|s| s.unique_players),
        5 => stats.sort_by_key(|s| s.wins),
        6 => stats.sort_by_key(|s| s.losses),
        23 => stats.sort_by_key(|s| s.roles.clone()),
        3 => stats.sort_by(|a, b| {
            a.pick_percentage
                .partial_cmp(&b.pick_percentage)
                .unwrap_or(Ordering::Equal)
        }),
        7 => stats.sort_by(|a, b| {
            a.win_percentage
                .partial_cmp(&b.win_percentage)
                .unwrap_or(Ordering::Equal)
        }),
        8 => stats.sort_by(|a, b| a.kills.partial_cmp(&b.kills).unwrap_or(Ordering::Equal)),
        9 => stats.sort_by(|a, b| a.deaths.partial_cmp(&b.deaths).unwrap_or(Ordering::Equal)),
        10 => stats.sort_by(|a, b| a.assists.partial_cmp(&b.assists).unwrap_or(Ordering::Equal)),
        11 => stats.sort_by(|a, b| a.kda.partial_cmp(&b.kda).unwrap_or(Ordering::Equal)),
        12 => stats.sort_by(|a, b| a.cs.partial_cmp(&b.cs).unwrap_or(Ordering::Equal)),
        13 => stats.sort_by(|a, b| a.csm.partial_cmp(&b.csm).unwrap_or(Ordering::Equal)),
        16 => stats.sort_by(|a, b| a.gold.partial_cmp(&b.gold).unwrap_or(Ordering::Equal)),
        17 => stats.sort_by(|a, b| a.goldm.partial_cmp(&b.goldm).unwrap_or(Ordering::Equal)),
        20 => stats.sort_by(|a, b| {
            a.kill_percentage
                .partial_cmp(&b.kill_percentage)
                .unwrap_or(Ordering::Equal)
        }),
        21 => stats.sort_by(|a, b| {
            a.kill_share
                .partial_cmp(&b.kill_share)
                .unwrap_or(Ordering::Equal)
        }),
        22 => stats.sort_by(|a, b| {
            a.gold_share
                .partial_cmp(&b.gold_share)
                .unwrap_or(Ordering::Equal)
        }),
        14 => stats.sort_by(|a, b| match (a.vs, b.vs) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }),
        15 => stats.sort_by(|a, b| match (a.vsm, b.vsm) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }),
        18 => stats.sort_by(|a, b| match (a.damage, b.damage) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }),
        19 => stats.sort_by(|a, b| match (a.damagem, b.damagem) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }),
        _ => {}
    }

    match app.champs_sort_dir {
        -1 => {
            stats.reverse();
        }
        _ => {}
    }

    let histories: &Vec<ChampionHistory> = match app
        .all_champs_state
        .selected()
        .and_then(|i| stats.get(i))
        .and_then(|stat| app.champs_history.get(&stat.champion))
    {
        Some(x) => x,
        None => &Vec::new(),
    };

    let rows = stats.iter().enumerate().map(|(i, stat)| {
        let colour = match i % 2 {
            0 => colours.normal_row_color,
            _ => colours.alt_row_color,
        };
        Row::from_iter(vec![
            Cell::from(Text::from((i + 1).to_string())),
            Cell::from(Text::from(stat.champion.clone())),
            Cell::from(
                Text::from(stat.games.to_string()).alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}%", stat.pick_percentage))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(stat.unique_players.to_string())
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(stat.wins.to_string()).alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(stat.losses.to_string()).alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}%", stat.win_percentage))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.kills))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.deaths))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.assists))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.kda)).alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.cs)).alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.csm)).alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(match stat.vs {
                    Some(d) => format!("{:.2}", d),
                    None => "-".to_string(),
                })
                .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(match stat.vsm {
                    Some(d) => format!("{:.2}", d),
                    None => "-".to_string(),
                })
                .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}k", stat.gold / 1000f64))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.goldm))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(match stat.damage {
                    Some(d) => format!("{:.0}k", d / 1000f64),
                    None => "-".to_string(),
                })
                .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(match stat.damagem {
                    Some(d) => format!("{:.0}", d),
                    None => "-".to_string(),
                })
                .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}%", stat.kill_percentage))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}%", stat.kill_share))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(Text::from(format!("{:.1}%", stat.gold_share))),
            Cell::from(Text::from(stat.roles.to_string())),
        ])
        .style(Style::new().fg(colours.row_fg).bg(colour))
    });

    let t = Table::new(
        rows,
        [
            Constraint::Max(4),
            Constraint::Max(12),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(7),
            Constraint::Max(7),
            Constraint::Max(7),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(6),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(4),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Min(1),
        ],
    )
    .header(header)
    .row_highlight_style(SELECTED_STYLE)
    .column_highlight_style(SELECTED_STYLE)
    .highlight_spacing(HighlightSpacing::Always)
    .block(block);

    let sublay = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Min(1)])
        .split(layout[1]);

    StatefulWidget::render(t, layout[0], buf, &mut app.all_champs_state);
    if histories.len() > 0 {
        draw_champion_history(buf, sublay[0], histories);
    }
}

pub fn draw_champion_history(buf: &mut Buffer, area: Rect, histories: &Vec<ChampionHistory>) {
    let colours = TableColors::new(&tailwind::ORANGE);

    let block = Block::default()
        .title("Champion stats")
        .borders(Borders::ALL)
        .style(Style::default());

    let header = [
        "Role",
        "Player",
        "Result",
        "Vs Champ",
        "Vs Player",
        "K",
        "D",
        "A",
        "KDA",
        // "CS",
        "CS/M",
        // "VS",
        "VS/M",
        // "Gold",
        "G/M",
        // "Damage",
        "DMG/M",
        "KPAR",
        "KS",
        "GS",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(Style::default().fg(colours.header_fg).bg(colours.header_bg))
    .height(1);

    let rows = histories.iter().enumerate().map(|(i, stat)| {
        let colour = match i % 2 {
            0 => colours.normal_row_color,
            _ => colours.alt_row_color,
        };
        Row::from_iter(vec![
            Cell::from(Text::from(stat.role.clone())),
            Cell::from(Text::from(stat.player.clone())),
            Cell::from(Text::from(stat.result.clone())),
            Cell::from(Text::from(stat.champion_vs.clone())),
            Cell::from(Text::from(stat.player_vs.clone())),
            Cell::from(
                Text::from(format!("{:.0}", stat.kills))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.0}", stat.deaths))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.0}", stat.assists))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}", stat.kda)).alignment(ratatui::layout::Alignment::Right),
            ),
            // Cell::from(
            //     Text::from(format!("{:.0}", stat.cs)).alignment(ratatui::layout::Alignment::Right),
            // ),
            Cell::from(
                Text::from(format!("{:.1}", stat.csm)).alignment(ratatui::layout::Alignment::Right),
            ),
            // Cell::from(
            //     Text::from(match stat.vs {
            //         Some(d) => format!("{:.0}", d),
            //         None => "-".to_string(),
            //     })
            //     .alignment(ratatui::layout::Alignment::Right),
            // ),
            Cell::from(
                Text::from(match stat.vsm {
                    Some(d) => format!("{:.2}", d),
                    None => "-".to_string(),
                })
                .alignment(ratatui::layout::Alignment::Right),
            ),
            // Cell::from(
            //     Text::from(format!("{:.1}k", stat.gold / 1000f64))
            //         .alignment(ratatui::layout::Alignment::Right),
            // ),
            Cell::from(
                Text::from(format!("{:.0}", stat.goldm))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            // Cell::from(
            //     Text::from(match stat.damage {
            //         Some(d) => format!("{:.0}k", d / 1000f64),
            //         None => "-".to_string(),
            //     })
            //     .alignment(ratatui::layout::Alignment::Right),
            // ),
            Cell::from(
                Text::from(match stat.damagem {
                    Some(d) => format!("{:.0}", d),
                    None => "-".to_string(),
                })
                .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}%", stat.kill_percentage))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(
                Text::from(format!("{:.1}%", stat.kill_share))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
            Cell::from(Text::from(format!("{:.1}%", stat.gold_share))),
        ])
        .style(Style::new().fg(colours.row_fg).bg(colour))
    });

    let t = Table::new(
        rows,
        [
            Constraint::Max(7),
            Constraint::Max(10),
            Constraint::Max(5),
            Constraint::Max(9),
            Constraint::Max(10),
            Constraint::Max(3),
            Constraint::Max(3),
            Constraint::Max(3),
            Constraint::Max(5),
            // Constraint::Max(4),
            Constraint::Max(4),
            // Constraint::Max(3),
            Constraint::Max(5),
            // Constraint::Max(6),
            Constraint::Max(5),
            // Constraint::Max(5),
            Constraint::Max(5),
            Constraint::Max(4),
            Constraint::Max(5),
            Constraint::Min(1),
        ],
    )
    .header(header)
    .row_highlight_style(SELECTED_STYLE)
    .column_highlight_style(SELECTED_STYLE)
    .highlight_spacing(HighlightSpacing::Always)
    .block(block);

    Widget::render(t, area, buf);
}
