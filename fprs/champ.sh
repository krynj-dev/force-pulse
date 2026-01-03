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

SELECT
    json_extract(p.value, '$.championName') AS name,
    json_extract(p.value, '$.teamPosition') AS role,
    CASE
        WHEN json_extract(p.value, '$.win') = 1 THEN 'WIN'
        ELSE 'LOSS'
    END AS result,
    json_extract(p.value, '$.riotIdGameName') AS player,
    date( json_extract(m.data, '$.info.gameEndTimestamp'), 'unixepoch' ) as date_
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.championName') = :name
ORDER BY json_extract(m.data, '$.info.gameEndTimestamp') DESC
LIMIT 50;
SQL

