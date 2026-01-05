sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column


SELECT *
FROM (
    SELECT
        CAST(json_extract(m.data, '$.info.gameId') AS TEXT) AS matchId,
        json_extract(t.value, '$.teamId') AS team,
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
      SUM(json_extract(p.value, '$.goldEarned')) AS gold
    FROM game m
    JOIN json_each(m.data, '$.info.teams') t
    JOIN json_each(m.data, '$.info.participants') p ON team=json_extract(p.value, '$.teamId')
    GROUP BY matchId, team
)
WHERE matchId = :name
ORDER BY team
LIMIT 20;

SQL

echo ''
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column


SELECT *
FROM (
    SELECT
        CAST(json_extract(m.data, '$.info.gameId') AS TEXT) AS matchId,
        CASE WHEN json_extract(t.value, '$.teamId')=100 THEN 'BLUE' ELSE 'RED' END AS team,
    json_extract(p.value, '$.riotIdGameName') AS player,
    json_extract(p.value, '$.teamPosition') AS role,
    json_extract(p.value, '$.championName') AS champ,
    json_extract(p.value, '$.kills')     AS kills,
    json_extract(p.value, '$.deaths')    AS deaths,
    json_extract(p.value, '$.assists')   AS assists,
    ROUND(
        (json_extract(p.value, '$.kills') +
         json_extract(p.value, '$.assists')) * 1.0 /
        CASE
            WHEN json_extract(p.value, '$.deaths') = 0 THEN 1
            ELSE json_extract(p.value, '$.deaths')
        END,
        2
    ) AS kda,
      json_extract(p.value, '$.goldEarned') AS gold,
( json_extract(p.value, '$.kills') + json_extract(p.value, '$.assists')   )* 100 / json_extract(t.value, '$.objectives.champion.kills') as KP
    FROM game m
    JOIN json_each(m.data, '$.info.teams') t
    JOIN json_each(m.data, '$.info.participants') p ON json_extract(t.value, '$.teamId')=json_extract(p.value, '$.teamId')
)
WHERE matchId = :name
ORDER BY team
LIMIT 20;

SQL
