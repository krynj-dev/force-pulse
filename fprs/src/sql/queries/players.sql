WITH
games AS (
    SELECT
        id,
        json_extract(data, '$.info.gameDuration') AS duration
    FROM game
),

participants AS (
    SELECT
        g.id AS game_id,
        json_extract(p.value, '$.riotIdGameName') AS name,
        NULLIF(json_extract(p.value, '$.riotIdTagline'), 'null') AS tag,
        json_extract(p.value, '$.teamId')         AS team_id,
        (CASE WHEN json_extract(p.value, '$.teamId') = 100 THEN g.team_1 ELSE g.team_2 END ) AS team_name,
        json_extract(p.value, '$.teamPosition')  AS role,
        json_extract(p.value, '$.soloKills')          AS solos,
        json_extract(p.value, '$.kills')          AS kills,
        json_extract(p.value, '$.deaths')         AS deaths,
        json_extract(p.value, '$.assists')        AS assists,
        json_extract(p.value, '$.goldEarned')     AS gold,
        json_extract(p.value, '$.visionScore')     AS vis,
    ( CASE WHEN json_extract(p.value, '$.visionScore') IS NULL THEN 0 ELSE json_extract(g.data, '$.info.gameDuration') END ) AS game_length_vis,
        json_extract(p.value, '$.totalDamageDealtToChampions') AS damage,
    ( CASE WHEN json_extract(p.value, '$.totalDamageDealtToChampions') IS NULL THEN 0 ELSE json_extract(g.data, '$.info.gameDuration') END ) AS game_length_dmg,
        CASE
    WHEN json_extract(p.value, '$.teamPosition') = 'JUNGLE' THEN
        CAST( json_extract(p.value, '$.challenges.jungleCsBefore10Minutes') AS INTEGER) 
      ELSE
       CAST( json_extract(p.value, '$.challenges.laneMinionsFirst10Minutes') AS INTEGER)
    END AS cs10,
        -- json_extract(p.value, '$.challenges.laneMinionsFirst10Minutes') + CAST(COALESCE( json_extract(p.value, '$.challenges.jungleCsBefore10Minutes'), 0 ) AS INTEGER)     AS cs10,
       (  json_extract(p.value, '$.totalMinionsKilled') +
    COALESCE( json_extract(p.value, '$.totalAllyJungleMinionsKilled'), 0 ) + COALESCE( json_extract(p.value, '$.totalEnemyJungleMinionsKilled'), 0 ))
    AS cs,
        CASE WHEN json_extract(p.value, '$.firstBloodAssist') = TRUE THEN 1 ELSE 0 END AS fba,
        CASE WHEN json_extract(p.value, '$.firstBloodKill') = TRUE THEN 1 ELSE 0 END AS fbk,
        CASE WHEN json_extract(p.value, '$.firstTowerAssist') = TRUE THEN 1 ELSE 0 END AS fta,
        CASE WHEN json_extract(p.value, '$.firstTowerKill') = TRUE THEN 1 ELSE 0 END AS ftk
    FROM game g
    JOIN json_each(g.data, '$.info.participants') p
),

teams AS (
    SELECT
        g.id AS game_id,
        json_extract(t.value, '$.teamId') AS team_id,
        json_extract(t.value, '$.objectives.champion.kills') AS ckill,
        CASE WHEN json_extract(t.value, '$.objectives.champion.first') = TRUE THEN 1 ELSE 0 END AS fb,
        CASE WHEN json_extract(t.value, '$.objectives.tower.first') = TRUE THEN 1 ELSE 0 END AS ft,
        CASE WHEN json_extract(t.value, '$.objectives.dragon.first') = TRUE THEN 1 ELSE 0 END AS fd,
        json_extract(t.value, '$.objectives.dragon.kills') AS td,
        json_extract(t.value, '$.objectives.horde.kills') AS grubs
    FROM game g
    JOIN json_each(g.data, '$.info.teams') t
),

team_damage AS (
    SELECT
        game_id,
        team_id,
        SUM(COALESCE( damage , 0)) AS team_dmg
    FROM participants
    GROUP BY game_id, team_id
),

cs_diff AS (
  SELECT
    p.game_id,
    p.name,
    (p.cs10 - opp.cs10) AS cd10
  FROM participants p
  JOIN participants opp
    ON p.game_id = opp.game_id
   AND p.role = opp.role
   AND p.team_id != opp.team_id
  -- GROUP BY p.game_id, p.name
),

enemy_team AS (
  SELECT
    game_id,
    CASE WHEN team_id = 100 THEN 200 ELSE 100 END AS team_id,
    ckill AS enemy_kills
  FROM teams
)

SELECT
        p.name AS riot_id,
        COALESCE(p.tag, "") AS tag_line,
       p.team_name AS team_name,
        p.role AS role,
        COUNT(*)                         AS games,
        SUM(p.kills)                     AS kills,
        SUM(p.deaths)                    AS deaths,
        SUM(p.assists)                   AS assists,
        ROUND(
            (SUM(p.kills) + SUM(p.assists)) * 1.0 /
            CASE WHEN SUM(p.deaths) = 0 THEN 1 ELSE SUM(p.deaths) END,
            2
        ) AS kda,
        SUM(p.gold) /
        (SUM(g.duration) / 60) AS gpm,
        ROUND(
            SUM(p.cs) * 1.0 /
            (SUM(g.duration) / 60.0),
            1
        ) AS cspm,
  ROUND( AVG(c.cd10), 1 ) AS cd10,

        100*( SUM(p.kills) + SUM(p.assists) )/SUM(t.ckill) AS kill_participation,
        100*( SUM(p.deaths) )/SUM(ot.enemy_kills) AS death_participation,
        CAST(SUM(COALESCE(p.vis, 0)) AS INTEGER)* 1.0/(SUM(p.game_length_vis)/60) as vpm,
        CAST(SUM(COALESCE(p.damage, 0)) AS INTEGER)/(SUM(p.game_length_dmg)/60) as dpm,
        -- 100 * p.dmg / td.team_dmg AS dmg_share,
        -- 100 * SUM(COALESCE( p.dmg, 0 )) / SUM(td.team_dmg) AS damage_share,
        SUM(p.fbk) AS fb_kills,
        SUM(p.fba) AS fb_assists,
        SUM(p.ftk) AS ft_kills,
        SUM(p.fta) AS ft_assists

      FROM participants p
      JOIN games g
        ON g.id = p.game_id
      JOIN teams t
        ON t.game_id = p.game_id
        AND t.team_id = p.team_id
JOIN enemy_team ot
  ON ot.game_id = p.game_id
 AND ot.team_id = p.team_id
      JOIN team_damage td
        ON td.game_id = p.game_id
        AND td.team_id = p.team_id
LEFT JOIN ( SELECT DISTINCT game_id, name, cd10 FROM cs_diff ) c ON c.name = p.name AND c.game_id = p.game_id
      WHERE ( ?1 IS NULL OR p.team_name = ?1) AND (?2 IS NULL OR p.role = ?2 )
      GROUP BY p.name, p.role
      HAVING ( ?3 IS NULL OR games >= CAST( ?3 AS INTEGER ) )

