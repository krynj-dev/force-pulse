sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

SELECT
    json_extract(p.value, '$.riotIdGameName') AS name,
    -- json_extract(p.value, '$.puuid')          AS puuid,
    -- json_extract(p.value, '$.teamPosition') AS role,
    COUNT(*)                                  AS games,
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
      ROUND(
      ( SUM(json_extract(p.value, '$.totalMinionsKilled'))  * 1.0)/
      ( SUM(json_extract(m.data, '$.info.gameDuration')) / 60)
      , 1)
      AS cspm,
      SUM(json_extract(p.value ,'$.totalDamageDealtToChampions')) /
      ( SUM(json_extract(m.data, '$.info.gameDuration')) / 60) as dpm
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value ,'$.totalDamageDealtToChampions') IS NOT NULL
GROUP BY name
-- , role
HAVING games >= 4
ORDER BY 
  dpm DESC,
  kda DESC
LIMIT 30;
SQL

