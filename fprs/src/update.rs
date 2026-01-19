use std::{fmt::format, path::PathBuf, result};

use color_eyre::eyre::{Result, eyre};
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use rusqlite::Connection;
use serde_json::Value;
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::{AlertType, App, CurrentScreen, Message, StatsTab},
    command::{APP_NAME, import_manual_matches},
    riot,
    sql::{
        repo,
        schema::{Game, PlayerChampionStats, PlayerDeepStats, PlayerRoleStats},
    },
    ui::GameList,
};

const ROLES: [&'static str; 5] = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

pub fn update(app: &mut App, msg: Message) -> Option<Message> {
    match msg {
        Message::NextTab => match app.current_screen {
            CurrentScreen::Stats => {
                app.next_stats_tab();
                match app.stats_tab {
                    StatsTab::Champion => {
                        app.all_champs_stats =
                            repo::stats_all_champions(&app.db_connection.as_ref().unwrap())
                                .unwrap();
                        app.champs_history =
                            repo::stats_champion_history(&app.db_connection.as_ref().unwrap())
                                .unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        },
        Message::PrevTab => match app.current_screen {
            CurrentScreen::Stats => {
                app.previous_stats_tab();
                match app.stats_tab {
                    StatsTab::Champion => {
                        app.all_champs_stats =
                            repo::stats_all_champions(&app.db_connection.as_ref().unwrap())
                                .unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        },
        Message::OpenAlert => {
            app.show_alert = true;
        }
        Message::CloseAlert => {
            app.alert_message.clear();
            app.alert_type = AlertType::None;
            app.show_alert = false;
        }
        Message::ReloadDatabaseGames => {
            app.db_games = GameList::from_iter(
                repo::all_games(app.db_connection.as_ref().unwrap()).unwrap_or(Vec::new()),
            );
            app.game_count = app.db_games.len() as i64;
        }
        Message::ListEnd => match app.current_screen {
            CurrentScreen::Start | CurrentScreen::Main => app.db_games.state.select_last(),
            CurrentScreen::Search => app.search_games.state.select_last(),
            CurrentScreen::Stats => match app.stats_tab {
                StatsTab::Player => app.players_table_state.select_last(),
                StatsTab::Champion => app.all_champs_state.select_last(),
                _ => {}
            },
            _ => {}
        },
        Message::ListStart => match app.current_screen {
            CurrentScreen::Start | CurrentScreen::Main => app.db_games.state.select_first(),
            CurrentScreen::Search => app.search_games.state.select_first(),
            CurrentScreen::Stats => match app.stats_tab {
                StatsTab::Player => app.players_table_state.select_first(),
                StatsTab::Champion => app.all_champs_state.select_first(),
                _ => {}
            },
            _ => {}
        },
        Message::ListDown => match app.current_screen {
            CurrentScreen::Start | CurrentScreen::Main => app.db_games.state.select_next(),
            CurrentScreen::Search => app.search_games.state.select_next(),
            CurrentScreen::Stats => match app.stats_tab {
                StatsTab::Player => app.players_table_state.select_next(),
                StatsTab::Champion => app.all_champs_state.select_next(),
                _ => {}
            },
            _ => {}
        },
        Message::ListUp => match app.current_screen {
            CurrentScreen::Start | CurrentScreen::Main => app.db_games.state.select_previous(),
            CurrentScreen::Search => app.search_games.state.select_previous(),
            CurrentScreen::Stats => match app.stats_tab {
                StatsTab::Player => app.players_table_state.select_previous(),
                StatsTab::Champion => app.all_champs_state.select_previous(),
                _ => {}
            },
            _ => {}
        },
        Message::ListLeft => match app.current_screen {
            CurrentScreen::Stats => match app.stats_tab {
                StatsTab::Player => app.players_table_state.select_previous_column(),
                StatsTab::Champion => app.all_champs_state.select_previous_column(),
                _ => {}
            },
            _ => {}
        },
        Message::ListRight => match app.current_screen {
            CurrentScreen::Stats => match app.stats_tab {
                StatsTab::Player => app.players_table_state.select_next_column(),
                StatsTab::Champion => app.all_champs_state.select_next_column(),
                _ => {}
            },
            _ => {}
        },
        Message::LoadGameCount => {
            app.game_count = repo::game_count(app.db_connection.as_ref()?).unwrap_or(0);
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
            if app.show_input {
                app.input.handle_event(&key);
                return None;
            }
            match app.current_screen {
                CurrentScreen::Stats => match key {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('r') => {
                            app.players_role_filter = (app.players_role_filter + 1) % 6;
                        }
                        KeyCode::Char('<') => match app.stats_tab {
                            StatsTab::Player => app.players_sort_dir = -1,
                            StatsTab::Champion => app.champs_sort_dir = -1,
                            _ => {}
                        },
                        KeyCode::Char('>') => match app.stats_tab {
                            StatsTab::Player => app.players_sort_dir = 1,
                            StatsTab::Champion => app.champs_sort_dir = 1,
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
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
                    app.alert_message = format!("Imported {} games", n);
                    app.alert_type = AlertType::Success;
                    return Some(Message::OpenAlert);
                }
                Err(e) => {
                    app.import_message = format!("Import failed: {}", e);
                    app.alert_message = format!("Import failed: {}", e);
                    app.alert_type = AlertType::Error;
                    return Some(Message::OpenAlert);
                }
            }
        }
        Message::OpenSearch => {
            app.previous_screen = app.current_screen.clone();
            app.next_screen = CurrentScreen::Search;
            app.post_message = Some(Message::DoSearch);
            return Some(Message::PromptInput);
        }
        Message::DoSearch => {
            let search_result = do_search(app);
            match search_result {
                Ok(matches) => {
                    app.alert_message = format!("Search successful!");
                    app.alert_type = AlertType::Success;
                    app.search_games = GameList::from_iter(matches.iter().map(|m| {
                        Game {
                            id: m
                                .get("metadata")
                                .and_then(|md| md.get("matchId"))
                                .and_then(|mid| mid.as_u64())
                                .unwrap_or(0),
                            data: m.clone(),
                            // manual: true,
                            team_1: "Unknown 1".to_string(),
                            team_2: "Unknown 2".to_string(),
                        }
                    }));
                    return Some(Message::OpenAlert);
                }
                Err(e) => {
                    app.alert_message = format!("Search failed: {e}");
                    app.alert_type = AlertType::Error;
                    app.search_games = GameList::default();
                    return Some(Message::OpenAlert);
                }
            }
        }
        Message::RemoveGame => {
            let game_id = app
                .db_games
                .state
                .selected()
                .map(|i| {
                    &app.db_games
                        .get_item(i)
                        .ok_or(eyre!("failed to get game obj"))
                        .unwrap()
                        .id
                })
                .ok_or(eyre!("faile to read game"))
                .unwrap();

            let del_result = repo::delete_game(&app.db_connection.as_ref().unwrap(), *game_id);
            match del_result {
                Ok(_) => return Some(Message::ReloadDatabaseGames),
                Err(e) => {
                    app.alert_message = format!("Failed to delete game: {e}");
                    app.alert_type = AlertType::Error;
                    app.search_games = GameList::default();
                    return Some(Message::OpenAlert);
                }
            }
        }
        Message::AddSearchGame => {
            let game_to_add = app
                .search_games
                .state
                .selected()
                .map(|i| {
                    &app.search_games
                        .get_item(i)
                        .ok_or(color_eyre::eyre::eyre!("failed to get game obj"))
                        .unwrap()
                        .data
                })
                .ok_or(eyre!("failed to read game"))
                .unwrap();
            let game_id = game_to_add
                .get("info")
                .and_then(|i| i.get("gameId"))
                .and_then(|id| id.as_u64())
                .ok_or(eyre!("Failed to get game ID"))
                .unwrap();
            let insert_result = repo::insert_game(
                &app.db_connection.as_ref().unwrap(),
                &game_id.to_string(),
                game_to_add,
            );
            match insert_result {
                Ok(_) => {
                    app.post_message = Some(Message::AddSearchTeam1);
                    return Some(Message::PromptInput);
                }
                Err(_) => {}
            }
        }
        Message::AddSearchTeam1 => {
            let team = app.messages.last()?;
            let game_to_add = app
                .search_games
                .state
                .selected()
                .map(|i| {
                    &app.search_games
                        .get_item(i)
                        .ok_or(color_eyre::eyre::eyre!("failed to get game obj"))
                        .unwrap()
                        .data
                })
                .ok_or(eyre!("failed to read game"))
                .unwrap();
            let game_id = game_to_add
                .get("info")
                .and_then(|i| i.get("gameId"))
                .and_then(|id| id.as_u64())
                .ok_or(eyre!("Failed to get game ID"))
                .unwrap();

            match repo::update_team(&app.db_connection.as_ref().unwrap(), game_id, team, 1) {
                Ok(_) => {
                    app.post_message = Some(Message::AddSearchTeam2);
                    return Some(Message::PromptInput);
                }
                Err(e) => {
                    app.alert_message = format!("Search failed: {e}");
                    app.alert_type = AlertType::Error;
                    app.search_games = GameList::default();
                    return Some(Message::OpenAlert);
                }
            }
        }
        Message::AddSearchTeam2 => {
            let team = app.messages.last()?;
            let game_to_add = app
                .search_games
                .state
                .selected()
                .map(|i| {
                    &app.search_games
                        .get_item(i)
                        .ok_or(color_eyre::eyre::eyre!("failed to get game obj"))
                        .unwrap()
                        .data
                })
                .ok_or(eyre!("failed to read game"))
                .unwrap();
            let game_id = game_to_add
                .get("info")
                .and_then(|i| i.get("gameId"))
                .and_then(|id| id.as_u64())
                .ok_or(eyre!("Failed to get game ID"))
                .unwrap();

            match repo::update_team(&app.db_connection.as_ref().unwrap(), game_id, team, 2) {
                Ok(_) => {
                    return Some(Message::ReloadDatabaseGames);
                }
                Err(e) => {
                    app.alert_message = format!("Search failed: {e}");
                    app.alert_type = AlertType::Error;
                    app.search_games = GameList::default();
                    return Some(Message::OpenAlert);
                }
            }
        }
        Message::OpenStats => {
            match repo::stats_overall(app.db_connection.as_ref().unwrap()) {
                Ok(result) => {
                    app.overall_stats = result;
                }
                Err(e) => {
                    app.alert_message = format!("Failed to load stats: {}", e);
                    app.alert_type = AlertType::Error;
                    app.show_alert = true;
                }
            }
            match repo::stats_players(
                app.db_connection.as_ref().unwrap(),
                [
                    None,
                    match app.players_role_filter {
                        0 => None,
                        i if i > 6 => None,
                        i => Some(ROLES[(i - 1) as usize].to_string()),
                    },
                    Some("4".to_string()),
                ],
            ) {
                Ok(result) => {
                    app.players_stats = result;
                    app.players_sort_dir = -1;
                    app.players_sort_col = 0;
                }
                Err(e) => {
                    app.alert_message = format!("Failed to load stats: {}", e);
                    app.alert_type = AlertType::Error;
                    app.show_alert = true;
                }
            }
            match (
                repo::stats_player_overall(app.db_connection.as_ref().unwrap()),
                repo::stats_player_role(app.db_connection.as_ref().unwrap()),
                repo::stats_player_champion(app.db_connection.as_ref().unwrap()),
            ) {
                (Ok(ov_result), Ok(ro_result), Ok(ch_result)) => {
                    app.player_deep_stats = PlayerDeepStats {
                        overall_stats: ov_result,
                        role_stats: ro_result,
                        champion_stats: ch_result,
                    }
                }
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                    app.alert_message = format!("Failed to load stats: {}", e);
                    app.alert_type = AlertType::Error;
                    app.show_alert = true;
                }
            }
            app.current_screen = CurrentScreen::Stats;
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

fn do_search(app: &App) -> color_eyre::Result<Vec<Value>> {
    let sea = "sea";
    let asia = "asia";
    // Get puuid
    let full_name = app
        .messages
        .last()
        .map(|s| s.as_str())
        .ok_or(color_eyre::eyre::eyre!("could not convert input to str"))?;
    let components: Vec<&str> = full_name.split('#').collect();
    if components.len() != 2 {
        return Err(eyre!(
            "failed to parse {} into form riotName#tagLine.",
            full_name
        ));
    }
    let riot_name = components
        .first()
        .ok_or(color_eyre::eyre::eyre!("could not find riot_name"))?;
    let tag_line = components
        .last()
        .ok_or(color_eyre::eyre::eyre!("could not find tag_line"))?;
    let acc = riot::fetch_account(&app.get_config_api_key(), &asia, riot_name, tag_line)?;
    let puuid = acc
        .get("puuid")
        .ok_or(color_eyre::eyre::eyre!("missing puuid"))?
        .as_str()
        .ok_or(color_eyre::eyre::eyre!("cannot convert puuid to str"))?;
    // Get match ids
    let match_ids_response = riot::fetch_match_ids(&app.get_config_api_key(), &sea, &puuid)?;
    let match_ids = match_ids_response
        .as_array()
        .ok_or(color_eyre::eyre::eyre!("could not read match id array"))?;
    let matches = match_ids
        .iter()
        .map(|mid| {
            let response =
                riot::fetch_match(&app.get_config_api_key(), &sea, mid.as_str().unwrap())?;
            Ok(response)
        })
        .collect::<color_eyre::Result<Vec<_>>>()?;
    Ok(matches)
}
