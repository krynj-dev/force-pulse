use std::collections::HashMap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    symbols::{
        border::{self, Set},
        line,
    },
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};
use serde_json::Value;

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

pub fn draw_scoreboard(buf: &mut Buffer, area: Rect, game_data: &Value) {
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
            "Team 1",
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
            "Team 2",
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
                " {:>width$} ",
                pad_to_width(player_name, col_widths[0], true),
                width = col_widths[0]
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
    let mut current_width = 0;
    let mut truncated = String::new();

    for c in s.chars() {
        let cw = c.width().unwrap_or(2);
        if current_width + cw > max_width.saturating_sub(1) {
            // leave space for '…'
            truncated.push('…');
            break;
        }
        truncated.push(c);
        current_width += cw;
    }

    truncated
}

pub fn pad_to_width(s: &str, width: usize, align_right: bool) -> String {
    let display_width = UnicodeWidthStr::width(s);
    if display_width >= width {
        // Already too wide, truncate with ellipsis
        let mut current_width = 0;
        let mut truncated = String::new();
        for c in s.chars() {
            let w = c.width().unwrap_or(0);
            if current_width + w > width.saturating_sub(1) {
                truncated.push('…');
                break;
            }
            truncated.push(c);
            current_width += w;
        }
        truncated
    } else {
        // Pad to reach width
        let pad = width - display_width;
        if align_right {
            format!("{:>width$}", s, width = s.len() + pad)
        } else {
            format!("{:<width$}", s, width = s.len() + pad)
        }
    }
}
