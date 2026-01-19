sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

SELECT
    ROW_NUMBER() OVER (ORDER BY 
        uplr DESC, games ASC) AS no,
    * FROM ( SELECT
         -- AS team_name,
    json_extract(p.value, '$.championName') AS name,
    -- json_extract(p.value, '$.puuid')          AS puuid,
    COUNT(*)                                  AS games,
  ROUND( COUNT(*) * 100.0 / (SELECT COUNT(*) from game ), 1) as pickr,
  COUNT (DISTINCT json_extract(p.value, '$.riotIdGameName')) as uplr,
    -- COUNT(json_extract(p.value, '$.win') = 'true')     AS wins,
    group_concat(DISTINCT json_extract(p.value,  '$.teamPosition' )) as roles, 
-- json_extract(p.value,  '$.teamPosition' ) AS role,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent
        FROM game m
JOIN json_each(m.data, '$.info.participants') p
-- WHERE (CASE WHEN json_extract(p.value, '$.teamId') = 100 THEN m.team_1 ELSE m.team_2 END ) = 'L Team'
GROUP BY name
-- HAVING games > 3
ORDER BY 
  uplr DESC,
  games ASC
LIMIT 50);
SQL
