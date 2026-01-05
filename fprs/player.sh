echo 'Overall'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.riotIdGameName') AS name,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent,
    SUM(json_extract(p.value, '$.kills'))     AS kills,
    SUM(json_extract(p.value, '$.deaths'))    AS deaths,
    SUM(json_extract(p.value, '$.assists'))   AS assists,
    ROUND(
        (SUM(json_extract(p.value, '$.kills')) +
         SUM(json_extract(p.value, '$.assists'))) * 1.0 /
        CASE
            WHEN SUM(json_extract(p.value, '$.deaths')) = 0 THEN 1
            ELSE SUM(json_extract(p.value, '$.deaths'))
        END,
    2
    ) AS kda,
    -- ROUND(
      SUM(json_extract(p.value, '$.goldEarned')) /
      ( SUM(json_extract(m.data, '$.info.gameDuration')) / 60)
      -- ,0)
      AS gpm,
      CAST(AVG(json_extract(p.value, '$.totalDamageDealtToChampions')) AS INTEGER) as damage,
        printf('%d:%02d', AVG(json_extract(m.data, '$.info.gameDuration')) / 60, AVG(json_extract(m.data, '$.info.gameDuration')) % 60) AS game_length
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name
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
    json_extract(p.value, '$.riotIdGameName') AS name,
    json_extract(p.value, '$.teamPosition') AS role,
    COUNT(*) AS games,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent,
    SUM(json_extract(p.value, '$.kills'))     AS kills,
    SUM(json_extract(p.value, '$.deaths'))    AS deaths,
    SUM(json_extract(p.value, '$.assists'))   AS assists,
    ROUND(
        (SUM(json_extract(p.value, '$.kills')) +
         SUM(json_extract(p.value, '$.assists'))) * 1.0 /
        CASE
            WHEN SUM(json_extract(p.value, '$.deaths')) = 0 THEN 1
            ELSE SUM(json_extract(p.value, '$.deaths'))
        END,
        2
    ) AS kda,
      SUM(json_extract(p.value, '$.goldEarned')) /
      ( SUM(json_extract(m.data, '$.info.gameDuration')) / 60)
      AS gpm,
      SUM(json_extract(p.value, '$.totalDamageDealtToChampions')) as damage
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name
GROUP BY role
ORDER BY games DESC, win_percent DESC
SQL

echo ''
echo 'Champions'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.riotIdGameName') AS name,
    json_extract(p.value, '$.championName') AS champion,
    COUNT(*) as games,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent,
    SUM(json_extract(p.value, '$.kills'))     AS kills,
    SUM(json_extract(p.value, '$.deaths'))    AS deaths,
    SUM(json_extract(p.value, '$.assists'))   AS assists,
    ROUND(
        (SUM(json_extract(p.value, '$.kills')) +
         SUM(json_extract(p.value, '$.assists'))) * 1.0 /
        CASE
            WHEN SUM(json_extract(p.value, '$.deaths')) = 0 THEN 1
            ELSE SUM(json_extract(p.value, '$.deaths'))
        END,
        2
    ) AS kda,
          SUM(json_extract(p.value, '$.goldEarned')) /
      ( SUM(json_extract(m.data, '$.info.gameDuration')) / 60)
      AS gpm,
      SUM(json_extract(p.value, '$.totalDamageDealtToChampions')) as damage

FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name
GROUP BY champion
ORDER BY games DESC, win_percent DESC
SQL

echo ''
echo 'Match history'
sqlite3 ~/.config/fprs/app.db <<SQL
.headers on
.mode column
.parameter init
.parameter set :name "$1"

WITH participants AS (
    SELECT
        g.id AS game_id,
        json_extract(p.value, '$.teamId') AS team_id,
        json_extract(p.value, '$.riotIdGameName') AS player,
        json_extract(p.value, '$.championName') AS champion,
        json_extract(p.value, '$.teamPosition') AS role,
        json_extract(p.value, '$.win') AS win,
        json_extract(p.value, '$.kills') AS kills,
        json_extract(p.value, '$.deaths') AS deaths,
        json_extract(p.value, '$.assists') AS assists,
        json_extract(p.value, '$.goldEarned') AS gold,
        json_extract(p.value, '$.totalDamageDealtToChampions') AS damage,
        json_extract(g.data, '$.info.gameDuration') AS game_duration,
        json_extract(g.data, '$.info.gameEndTimestamp') AS game_end
    FROM game g
    JOIN json_each(g.data, '$.info.participants') p
    WHERE json_extract(p.value, '$.riotIdGameName') = :name
),

team_totals AS (
    SELECT
        g.id AS game_id,
        json_extract(p.value, '$.teamId') AS team_id,
        SUM(json_extract(p.value, '$.kills')) AS team_kills
    FROM game g
    JOIN json_each(g.data, '$.info.participants') p
    GROUP BY game_id, team_id
)

SELECT
    player,
    champion,
    role,
    CASE WHEN win=1 THEN 'WIN' ELSE 'LOSS' END AS result,
    kills,
    deaths,
    assists,
    ROUND((kills + assists) * 1.0 / CASE WHEN deaths=0 THEN 1 ELSE deaths END, 2) AS kda,
    gold / (game_duration / 60) AS gpm,
    damage,
    damage / (game_duration / 60) AS dpm,
    printf('%d:%02d', game_duration / 60, game_duration % 60) AS len,
    t.team_kills as tk
FROM participants p
JOIN team_totals t
  ON p.game_id = t.game_id
 AND p.team_id = t.team_id
ORDER BY game_end DESC
LIMIT 50;

SQL

