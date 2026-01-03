sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

SELECT
    COUNT(*)/2                                  AS games,
    SUM(CASE WHEN json_extract(t.value, '$.win') = true AND json_extract(t.value, '$.teamId') = 100 THEN 1 ELSE 0 END) AS blue_wins,
    SUM(CASE WHEN json_extract(t.value, '$.win') = true AND json_extract(t.value, '$.teamId') = 200 THEN 1 ELSE 0 END) AS red_wins,
    -- AVG(json_extract(m.data, '$.info.gameDuration')) as game_length
        printf('%d:%02d', AVG(json_extract(m.data, '$.info.gameDuration')) / 60, AVG(json_extract(m.data, '$.info.gameDuration')) % 60) AS avg_game_length,
        printf('%d:%02d', MIN(json_extract(m.data, '$.info.gameDuration')) / 60, MIN(json_extract(m.data, '$.info.gameDuration')) % 60) AS min_game_length,
        printf('%d:%02d', MAX(json_extract(m.data, '$.info.gameDuration')) / 60, MAX(json_extract(m.data, '$.info.gameDuration')) % 60) AS max_game_length
        FROM game m
JOIN json_each(m.data, '$.info.teams') t
SQL
