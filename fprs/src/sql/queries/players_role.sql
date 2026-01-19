WITH participants AS (
    SELECT
        g.id AS game_id,
        json_extract(p.value, '$.riotIdGameName') AS player_name,
        json_extract(p.value, '$.championName') AS champion,
        json_extract(p.value, '$.teamPosition') AS role,
        json_extract(p.value, '$.win') AS win,
        json_extract(p.value, '$.teamId') AS team_id,
        json_extract(p.value, '$.kills') AS kills,
        json_extract(p.value, '$.deaths') AS deaths,
        json_extract(p.value, '$.assists') AS assists,
        json_extract(p.value, '$.goldEarned') AS gold,
        json_extract(p.value, '$.totalDamageDealtToChampions') AS damage,
    ( CASE WHEN json_extract(p.value, '$.totalDamageDealtToChampions') IS NULL THEN 0 ELSE json_extract(g.data, '$.info.gameDuration') END ) AS game_length_dmg,
        COALESCE( json_extract(p.value, '$.challenges.laneMinionsFirst10Minutes'), 0 ) + CAST(COALESCE( json_extract(p.value, '$.challenges.jungleCsBefore10Minutes') , 0) AS INTEGER)     AS cs10,
        json_extract(g.data, '$.info.gameDuration') AS game_length

    FROM game g
    JOIN json_each(g.data, '$.info.participants') p
),
cs_diff AS (
  SELECT
    p.game_id,
    p.player_name,
    AVG(p.cs10 - opp.cs10) AS cd10
  FROM participants p
  JOIN participants opp
    ON p.game_id = opp.game_id
   AND p.role = opp.role
   AND p.team_id != opp.team_id
  GROUP BY p.game_id, p.player_name
)


SELECT
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY kills_per_game DESC) AS kpgn, 
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY deaths_per_game ASC) AS dpgn,
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY assists_per_game DESC) AS apgn, 
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY kda DESC) AS kdan, 
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY gpm DESC) AS gpmn, 
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY dpm DESC) AS dpmn, 
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY cd10 DESC) AS cd10n, 
  ROW_NUMBER() OVER (PARTITION BY role ORDER BY game_length DESC) AS gln, 
  COUNT(*) OVER (PARTITION BY role) as role_total,
  * FROM ( SELECT
  p.player_name as player_name,
  p.role as role,
  COUNT(*) as games,
    SUM(CASE WHEN p.win= true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN p.win= false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN p.win = true THEN 1 ELSE 0 END) / COUNT(*)  as win_percent,
    SUM(p.kills)/COUNT(*)     AS kills_per_game,
    SUM(p.deaths)/COUNT(*)    AS deaths_per_game,
    SUM(p.assists)/COUNT(*)   AS assists_per_game,
    ROUND(
        (SUM(p.kills) +
         SUM(p.assists)) * 1.0 /
        CASE
            WHEN SUM(p.deaths) = 0 THEN 1
            ELSE SUM(p.deaths)
        END,
    2
    ) AS kda,
      SUM(p.gold) /
      ( SUM(p.game_length) / 60)
      AS gpm,
      CAST( CAST(SUM(COALESCE( p.damage, 0 )) AS INTEGER)/       ( SUM(p.game_length_dmg) / 60) AS INTEGER ) as dpm,
ROUND( AVG(c.cd10) , 1) AS cd10,
        printf('%d:%02d', AVG(p.game_length) / 60, AVG(p.game_length) % 60) AS game_length
FROM participants p LEFT JOIN cs_diff c ON c.player_name = p.player_name AND c.game_id = p.game_id
GROUP BY p.player_name, p.role HAVING games >= 4 
ORDER BY games DESC, win_percent DESC 
 );
