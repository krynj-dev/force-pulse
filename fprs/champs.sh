sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

SELECT
    json_extract(p.value, '$.championName') AS name,
    -- json_extract(p.value, '$.puuid')          AS puuid,
    COUNT(*)                                  AS games,
    -- COUNT(json_extract(p.value, '$.win') = 'true')     AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent
        FROM game m
JOIN json_each(m.data, '$.info.participants') p
GROUP BY name
ORDER BY 
  games DESC, win_percent DESC
LIMIT 15;
SQL
