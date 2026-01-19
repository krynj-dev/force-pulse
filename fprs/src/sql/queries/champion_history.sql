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
  p.champion as champion,
  p2.champion as champion_vs,
  p.role as role,
  CASE p.win
    WHEN 1 THEN 'WIN'
    WHEN 0 THEN 'LOSS'
    ELSE '-'
  END as result,
  p.player_name as player,
  p2.player_name as player_vs,
  p.game_length as game_length,
  p.kills as kills,
  p.deaths as deaths,
  p.assists as assists,
  (p.kills + p.assists)*1.0/(CASE p.deaths WHEN 0 THEN 1 ELSE p.deaths END) as kda,
  p.cs as cs,
  p.cs*1.0 / (p.game_length / 60.0) AS csm,
  p.vis as vs,
  p.vis*1.0 / (p.game_length_vis / 60.0) AS vsm,
  p.gold as gold,
  p.gold*1.0 / (p.game_length / 60.0) AS gpm,
  p.damage as damage,
  p.damage*1.0 / (p.game_length_dmg / 60.0) as dpm,
  (p.kills+p.assists)*100.0/ t.kills as kill_percentage,
  (p.kills)*100.0/ t.kills as kill_share,
  (p.gold)*100.0/ t.gold as gold_share

FROM participants p JOIN teams t ON p.game_id = t.game_id AND p.team_id = t.team_id 
JOIN participants p2 on p2.game_id=p.game_id AND p.role=p2.role AND p.player_name != p2.player_name
ORDER BY p.game_id DESC;

