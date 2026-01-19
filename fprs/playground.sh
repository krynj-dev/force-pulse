
sqlite3 ~/.config/fprs/app.db <<SQL
.headers on
.mode column

WITH
teams AS (
  SELECT
    g.id AS game_id,
    json_extract(p.value, '$.teamId') as team_id,
    SUM(json_extract(p.value, '$.kills') ) AS kills,
    SUM(json_extract(p.value, '$.deaths') ) AS deaths,
    SUM(json_extract(p.value, '$.goldEarned') ) AS gold
    -- json_extract(p.value, '$.totalDamageDealtToChampions') AS damage
  FROM game g JOIN json_each(g.data, '$.info.participants') p
  GROUP BY game_id, team_id
),
participants AS (
  SELECT
        g.id AS game_id,
        json_extract(p.value, '$.riotIdGameName') AS player_name,
        json_extract(p.value, '$.championName') AS champion,
        json_extract(p.value, '$.teamPosition') AS role,
        CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END AS win,
        CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END AS loss,
        json_extract(p.value, '$.teamId') AS team_id,
        json_extract(p.value, '$.kills') AS kills,
        json_extract(p.value, '$.deaths') AS deaths,
        json_extract(p.value, '$.assists') AS assists,
        json_extract(p.value, '$.goldEarned') AS gold,
        json_extract(p.value, '$.totalDamageDealtToChampions') AS damage,
        (json_extract(p.value, '$.totalMinionsKilled') +
          COALESCE( json_extract(p.value, '$.totalAllyJungleMinionsKilled'), 0 ) +
          COALESCE( json_extract(p.value, '$.totalEnemyJungleMinionsKilled'), 0 )
        ) AS cs,
        (CASE
          WHEN json_extract(p.value, '$.totalDamageDealtToChampions') IS NULL THEN 0 
          ELSE json_extract(g.data, '$.info.gameDuration')
        END) AS game_length_dmg,
        json_extract(p.value, '$.visionScore')     AS vis,
        (CASE
          WHEN json_extract(p.value, '$.visionScore') IS NULL THEN 0 
          ELSE json_extract(g.data, '$.info.gameDuration')
        END) AS game_length_vis,
        json_extract(g.data, '$.info.gameDuration') AS game_length

    FROM game g
    JOIN json_each(g.data, '$.info.participants') p
)

SELECT
  p.champion,
  COUNT(DISTINCT p.game_id) as games,
  COUNT(DISTINCT p.game_id)*100.0/(SELECT COUNT(*) from game) as pick_percentage,
  COUNT(DISTINCT p.player_name) as unique_players,
  SUM(p.win) AS wins,
  SUM(p.loss) AS losses,
  SUM(p.win) / (COUNT(DISTINCT p.game_id)) as win_percent,
  AVG(p.kills) as kills_avg,
  AVG(p.deaths) as deaths_avg,
  AVG(p.assists) as assists_avg,
  (SUM(p.kills)+SUM(p.assists)) * 1.0 / 
  (CASE
    WHEN SUM(p.deaths) = 0 THEN 1 ELSE SUM(p.deaths)
  END) as kda,
  AVG(p.cs) as cs_avg,
  SUM(p.cs)*1.0 / SUM(p.game_length) AS cs_min,
  AVG(p.vis) as vis_avg,
  SUM(p.vis)*1.0 / SUM(p.game_length_vis) AS vs_min,
  AVG(p.gold) as gold_avg,
  SUM(p.gold)*1.0 / SUM(p.game_length) AS gold_min,
  AVG(p.damage) AS damage_avg,
  SUM(p.damage)*1.0 / SUM(p.game_length_dmg) AS dmg_min,
  SUM(p.kills+p.assists)*1.0 / SUM(t.kills) AS kill_percent,
  SUM(p.kills)*1.0 / SUM(t.kills) AS kill_share,
  SUM(p.gold)*1.0 / SUM(t.gold) AS gold_share,
  group_concat(DISTINCT p.role) as roles
FROM participants p JOIN teams t ON p.game_id = t.game_id AND p.team_id = t.team_id
GROUP BY champion LIMIT 20
SQL
