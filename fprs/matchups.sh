sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

WITH
game_result as (
SELECT
 game.id as game_id,
  team_1,
  team_2,
  json_extract(t.value, '$.teamId') as team_id,
  json_extract(t.value, '$.win') as win
  FROM game join json_each(game.data, '$.info.teams') t
),

winner AS (
    SELECT
        game_id,
        team_1,
        team_2,
        CASE team_id
            WHEN 100 THEN team_1
            WHEN 200 THEN team_2
        END AS winning_team
    FROM game_result
    WHERE win = 1
),

normalized AS (
    SELECT
        MIN(team_1, team_2) AS team_a,
        MAX(team_1, team_2) AS team_b,
        winning_team
    FROM winner
)

SELECT
    team_a,
    SUM(winning_team = team_a) || '-' ||
    SUM(winning_team = team_b) AS series,
    team_b
FROM normalized
GROUP BY team_a, team_b
HAVING COUNT(*) BETWEEN 2 AND 3
ORDER BY team_a, team_b;
SQL

sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

WITH
game_result as (
SELECT
 game.id as game_id,
  team_1,
  team_2,
  json_extract(t.value, '$.teamId') as team_id,
  json_extract(t.value, '$.win') as win
  FROM game join json_each(game.data, '$.info.teams') t
),

winner AS (
    SELECT
        game_id,
        team_1,
        team_2,
        CASE team_id
            WHEN 100 THEN team_1
            WHEN 200 THEN team_2
        END AS winning_team
    FROM game_result
    WHERE win = 1
),

series AS (
    SELECT
        MIN(team_1, team_2) AS team_a,
        MAX(team_1, team_2) AS team_b,
        SUM(winning_team = MIN(team_1, team_2)) AS team_a_wins,
        SUM(winning_team = MAX(team_1, team_2)) AS team_b_wins,
        COUNT(*) AS games_played
    FROM winner
    GROUP BY team_a, team_b
    HAVING games_played BETWEEN 2 AND 3
),

per_team AS (
    SELECT
        team_a AS team,
        team_a_wins AS game_wins,
        team_b_wins AS game_losses,
        CASE WHEN team_a_wins > team_b_wins THEN 1 ELSE 0 END AS match_wins,
        CASE WHEN team_a_wins < team_b_wins THEN 1 ELSE 0 END AS match_losses
    FROM series

    UNION ALL

    SELECT
        team_b,
        team_b_wins,
        team_a_wins,
        CASE WHEN team_b_wins > team_a_wins THEN 1 ELSE 0 END,
        CASE WHEN team_b_wins < team_a_wins THEN 1 ELSE 0 END
    FROM series
)

SELECT
  ROW_NUMBER() OVER () as ranking, * FROM (
SELECT
    team,
    -- SUM(match_wins)   AS matches_won,
    -- SUM(match_losses) AS matches_lost,
    -- SUM(game_wins)    AS games_won,
    -- SUM(game_losses)  AS games_lost,
    SUM(match_wins) || '-' || SUM(match_losses) AS match_record,
    SUM(game_wins) || '-' || SUM(game_losses) AS game_record,
    SUM(match_wins)*100 / (SUM(match_wins) + SUM(match_losses)) AS match_wr,
    SUM(game_wins)*100 / (SUM(game_wins) + SUM(game_losses)) AS game_wr
FROM per_team
GROUP BY team
ORDER BY
    match_wr DESC,
    SUM(match_wins) DESC,
    game_wr DESC,
    SUM(game_wins) DESC
    );
SQL


