echo 'Overall'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.championName') AS name,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.championName') = :name
ORDER BY win_percent
SQL

echo ''
echo 'By role'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.championName') AS name,
    json_extract(p.value, '$.teamPosition') AS role,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.championName') = :name
GROUP BY json_extract(p.value, '$.teamPosition')
ORDER BY win_percent
SQL

echo ''
echo 'Match history'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

WITH participants AS (
    SELECT
        g.id AS game_id,
        json_extract(p.value, '$.championName') AS champion,
        json_extract(p.value, '$.teamPosition') AS role,
        json_extract(p.value, '$.riotIdGameName') AS player,
        json_extract(p.value, '$.win') AS win,
        json_extract(p.value, '$.teamId') AS team_id,
        json_extract(g.data, '$.info.gameEndTimestamp') AS game_end
    FROM game g
    JOIN json_each(g.data, '$.info.participants') p
),

player_wr AS (
    SELECT
        player,
        100 * SUM(CASE WHEN win=1 THEN 1 ELSE 0 END) / COUNT(*) AS win_percent
    FROM participants
    GROUP BY player
),

vs_champs AS (
    SELECT
        p1.game_id,
        p1.role,
        p1.player,
        p2.player AS vs_player,
        p2.champion AS vs_champ,
        vwr.win_percent AS vs_wr
    FROM participants p1
    JOIN participants p2
      ON p1.game_id = p2.game_id
     AND p1.role = p2.role
     AND p1.team_id != p2.team_id
    JOIN player_wr vwr
     ON p2.player=vwr.player
)

SELECT
    p.role,
    wr.win_percent AS pwr,
    p.player,
    p.champion AS name,
    CASE WHEN p.win = 1 THEN 'WIN' ELSE 'LOSS' END AS result,
    vc.vs_champ,
    vc.vs_player AS vs_plr,
    vc.vs_wr AS vwp
FROM participants p
JOIN player_wr wr
  ON p.player = wr.player
JOIN vs_champs vc
  ON p.player = vc.player
 AND p.game_id = vc.game_id
WHERE p.champion = :name
ORDER BY
    wr.win_percent DESC,
    p.game_end DESC
LIMIT 50;
SQL

