use std::collections::HashMap;

use rusqlite::Row;
use serde_json::Value;

pub struct Game {
    pub id: u64,
    pub team_1: String,
    pub team_2: String,
    // pub manual: bool,
    pub data: Value,
}

impl TryFrom<&Row<'_>> for Game {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let data_str = row.get::<_, String>("data")?;
        Ok(Self {
            id: row.get("id")?,
            team_1: row.get("team_1")?,
            team_2: row.get("team_2")?,
            // manual: row.get("manual")?,
            data: serde_json::from_str(&data_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        })
    }
}

#[derive(Debug, Default)]
pub struct OverallStats {
    pub games: u64,
    pub blue_wins: u64,
    pub red_wins: u64,
    pub game_length_avg: u64,
    pub game_length_min: u64,
    pub game_length_max: u64,
}

impl TryFrom<&Row<'_>> for OverallStats {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            games: row.get("games")?,
            blue_wins: row.get("blue_wins")?,
            red_wins: row.get("red_wins")?,
            game_length_avg: row.get("game_length_avg")?,
            game_length_min: row.get("game_length_min")?,
            game_length_max: row.get("game_length_max")?,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChampionHistory {
    pub champion: String,
    pub champion_vs: String,
    pub role: String,
    pub result: String,
    pub player: String,
    pub player_vs: String,
    pub game_length: u64,
    pub kills: f64,
    pub deaths: f64,
    pub assists: f64,
    pub kda: f64,
    pub cs: f64,
    pub csm: f64,
    pub vs: Option<f64>,
    pub vsm: Option<f64>,
    pub gold: f64,
    pub goldm: f64,
    pub damage: Option<f64>,
    pub damagem: Option<f64>,
    pub kill_percentage: f64,
    pub kill_share: f64,
    pub gold_share: f64,
}

impl TryFrom<&Row<'_>> for ChampionHistory {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            champion: row.get("champion")?,
            champion_vs: row.get("champion_vs")?,
            role: row.get("role")?,
            result: row.get("result")?,
            player: row.get("player")?,
            player_vs: row.get("player_vs")?,
            game_length: row.get("game_length")?,
            kills: row.get("kills")?,
            deaths: row.get("deaths")?,
            assists: row.get("assists")?,
            kda: row.get("kda")?,
            cs: row.get("cs")?,
            csm: row.get("csm")?,
            vs: row.get("vs")?,
            vsm: row.get("vsm")?,
            gold: row.get("gold")?,
            goldm: row.get("gpm")?,
            damage: row.get("damage")?,
            damagem: row.get("dpm")?,
            kill_percentage: row.get("kill_percentage")?,
            kill_share: row.get("kill_share")?,
            gold_share: row.get("gold_share")?,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChampionStats {
    pub champion: String,
    pub games: u64,
    pub pick_percentage: f64,
    pub unique_players: u64,
    pub wins: u64,
    pub losses: u64,
    pub win_percentage: f64,
    pub kills: f64,
    pub deaths: f64,
    pub assists: f64,
    pub kda: f64,
    pub cs: f64,
    pub csm: f64,
    pub vs: Option<f64>,
    pub vsm: Option<f64>,
    pub gold: f64,
    pub goldm: f64,
    pub damage: Option<f64>,
    pub damagem: Option<f64>,
    pub kill_percentage: f64,
    pub kill_share: f64,
    pub gold_share: f64,
    pub roles: String,
}

impl TryFrom<&Row<'_>> for ChampionStats {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            champion: row.get("champion")?,
            games: row.get("games")?,
            pick_percentage: row.get("pick_percentage")?,
            unique_players: row.get("unique_players")?,
            wins: row.get("wins")?,
            losses: row.get("losses")?,
            win_percentage: row.get("win_percentage")?,
            kills: row.get("kills")?,
            deaths: row.get("deaths")?,
            assists: row.get("assists")?,
            kda: row.get("kda")?,
            cs: row.get("cs")?,
            csm: row.get("csm")?,
            vs: row.get("vs")?,
            vsm: row.get("vsm")?,
            gold: row.get("gold")?,
            goldm: row.get("goldm")?,
            damage: row.get("damage")?,
            damagem: row.get("damagem")?,
            kill_percentage: row.get("kill_percentage")?,
            kill_share: row.get("kill_share")?,
            gold_share: row.get("gold_share")?,
            roles: row.get("roles")?,
        })
    }
}
#[derive(Debug, Default, Clone)]
pub struct PlayerStats {
    pub riot_id: String,
    pub tag_line: String,
    pub team_name: String,
    pub role: String,
    pub games: u64,
    pub kills: u64,
    pub deaths: u64,
    pub assists: u64,
    pub kda: f64,
    pub gpm: u64,
    pub cspm: f64,
    pub cd10: Option<f64>,
    pub kill_participation: u64,
    pub death_participation: u64,
    pub dpm: Option<u64>,
    pub vpm: Option<f64>,
    // First blood
    pub fb_kills: u64,
    pub fb_assists: u64,
    // First tower
    pub ft_kills: u64,
    pub ft_assists: u64,
}

impl TryFrom<&Row<'_>> for PlayerStats {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            riot_id: row.get("riot_id")?,
            tag_line: row.get("tag_line")?,
            team_name: row.get("team_name")?,
            role: row.get("role")?,
            games: row.get("games")?,
            kills: row.get("kills")?,
            deaths: row.get("deaths")?,
            assists: row.get("assists")?,
            kda: row.get("kda")?,
            gpm: row.get("gpm")?,
            cspm: row.get("cspm")?,
            cd10: row.get("cd10")?,
            vpm: row.get("vpm")?,
            kill_participation: row.get("kill_participation")?,
            death_participation: row.get("death_participation")?,
            dpm: row.get("dpm")?,
            fb_kills: row.get("fb_kills")?,
            fb_assists: row.get("fb_assists")?,
            ft_kills: row.get("ft_kills")?,
            ft_assists: row.get("ft_assists")?,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct PlayerChampionStats {
    pub player_name: String,
    pub champion_name: String,
    pub role: String,
    pub games: u64,
    pub wins: u64,
    pub losses: u64,
    pub win_percent: u64,
    pub kills_per_game: f64,
    pub deaths_per_game: f64,
    pub assists_per_game: f64,
    pub kda: f64,
    pub gpm: u64,
    pub dpm: Option<u64>,
    pub cd10: Option<f64>,
}

#[derive(Debug, Default, Clone)]
pub struct PlayerOverallStats {
    pub player_name: String,
    pub games: u64,
    pub wins: u64,
    pub losses: u64,
    pub win_percent: u64,
    pub kills_per_game: f64,
    pub deaths_per_game: f64,
    pub assists_per_game: f64,
    pub kda: f64,
    pub gpm: u64,
    pub dpm: Option<u64>,
    pub cd10: Option<f64>,
    pub total: u64,
    pub kpgn: u64,
    pub dpgn: u64,
    pub apgn: u64,
    pub kdan: u64,
    pub gpmn: u64,
    pub dpmn: Option<u64>,
    pub cd10n: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct PlayerRoleStats {
    pub player_name: String,
    pub role: String,
    pub games: u64,
    pub wins: u64,
    pub losses: u64,
    pub win_percent: u64,
    pub kills_per_game: f64,
    pub deaths_per_game: f64,
    pub assists_per_game: f64,
    pub kda: f64,
    pub gpm: u64,
    pub dpm: Option<u64>,
    pub cd10: Option<f64>,
    pub role_total: u64,
    pub kpgn: u64,
    pub dpgn: u64,
    pub apgn: u64,
    pub kdan: u64,
    pub gpmn: u64,
    pub dpmn: Option<u64>,
    pub cd10n: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct PlayerDeepStats {
    pub role_stats: HashMap<String, Vec<PlayerRoleStats>>,
    pub overall_stats: HashMap<String, PlayerOverallStats>,
    pub champion_stats: HashMap<String, Vec<PlayerChampionStats>>,
}

impl TryFrom<&Row<'_>> for PlayerOverallStats {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            player_name: row.get("player_name")?,
            games: row.get("games")?,
            wins: row.get("wins")?,
            losses: row.get("losses")?,
            win_percent: row.get("win_percent")?,
            kills_per_game: row.get("kills_per_game")?,
            deaths_per_game: row.get("deaths_per_game")?,
            assists_per_game: row.get("assists_per_game")?,
            kda: row.get("kda")?,
            gpm: row.get("gpm")?,
            dpm: row.get("dpm")?,
            cd10: row.get("cd10")?,
            total: row.get("total")?,
            kpgn: row.get("kpgn")?,
            dpgn: row.get("dpgn")?,
            apgn: row.get("apgn")?,
            kdan: row.get("kdan")?,
            gpmn: row.get("gpmn")?,
            dpmn: row.get("dpmn")?,
            cd10n: row.get("cd10n")?,
        })
    }
}

impl TryFrom<&Row<'_>> for PlayerChampionStats {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            player_name: row.get("player_name")?,
            role: row.get("role")?,
            champion_name: row.get("champion")?,
            games: row.get("games")?,
            wins: row.get("wins")?,
            losses: row.get("losses")?,
            win_percent: row.get("win_percent")?,
            kills_per_game: row.get("kills_per_game")?,
            deaths_per_game: row.get("deaths_per_game")?,
            assists_per_game: row.get("assists_per_game")?,
            kda: row.get("kda")?,
            gpm: row.get("gpm")?,
            dpm: row.get("dpm")?,
            cd10: row.get("cd10")?,
        })
    }
}

impl TryFrom<&Row<'_>> for PlayerRoleStats {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            player_name: row.get("player_name")?,
            role: row.get("role")?,
            games: row.get("games")?,
            wins: row.get("wins")?,
            losses: row.get("losses")?,
            win_percent: row.get("win_percent")?,
            kills_per_game: row.get("kills_per_game")?,
            deaths_per_game: row.get("deaths_per_game")?,
            assists_per_game: row.get("assists_per_game")?,
            kda: row.get("kda")?,
            gpm: row.get("gpm")?,
            dpm: row.get("dpm")?,
            cd10: row.get("cd10")?,
            role_total: row.get("role_total")?,
            kpgn: row.get("kpgn")?,
            dpgn: row.get("dpgn")?,
            apgn: row.get("apgn")?,
            kdan: row.get("kdan")?,
            gpmn: row.get("gpmn")?,
            dpmn: row.get("dpmn")?,
            cd10n: row.get("cd10n")?,
        })
    }
}
