sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT 
    json_extract(m.data, '$.info.gameId') AS game_id,
    json_group_array(json_extract(p.value, '$.riotIdGameName')) AS teammates
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE EXISTS (
    SELECT 1
    FROM json_each(m.data, '$.info.participants') p2
    WHERE json_extract(p2.value, '$.riotIdGameName') = :name
      AND json_extract(p.value, '$.teamId') = json_extract(p2.value, '$.teamId')
      AND json_extract(p.value, '$.riotIdGameName') != :name
)
GROUP BY m.id;
SQL

