sqlite3 ~/.config/fprs/app.db <<SQL
.headers on
.mode column
.parameter set :name "$1"

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
        json_extract(p.value, '$.teamId')         AS team_id,
        json_extract(p.value, '$.teamPosition')  AS role,
        json_extract(p.value, '$.kills')          AS kills,
        json_extract(p.value, '$.deaths')         AS deaths,
        json_extract(p.value, '$.assists')        AS assists,
        json_extract(p.value, '$.goldEarned')     AS gold,
        json_extract(p.value, '$.totalDamageDealtToChampions')     AS dmg,
        json_extract(p.value, '$.totalMinionsKilled') AS cs
    FROM game g
    JOIN json_each(g.data, '$.info.participants') p
),

teams AS (
    SELECT
        g.id AS game_id,
        json_extract(t.value, '$.teamId') AS team_id,
        json_extract(t.value, '$.objectives.champion.kills') AS ckill
    FROM game g
    JOIN json_each(g.data, '$.info.teams') t
),

team_damage AS (
    SELECT
        game_id,
        team_id,
        SUM(dmg) AS team_dmg
    FROM participants
    WHERE dmg IS NOT NULL
    GROUP BY game_id, team_id
)

SELECT
    ROW_NUMBER() OVER (ORDER BY 
    -- dmg_share ASC,
    dp DESC,
    cspm DESC,
    kp DESC,
    gpm DESC
    ) AS no,
    * FROM (
      SELECT 
        p.name AS name,
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
        100*( SUM(p.kills) + SUM(p.assists) )/SUM(t.ckill) AS kp,
        100*( SUM(p.deaths) )/SUM(ot.ckill) AS dp
        -- 100 * p.dmg / td.team_dmg AS dmg_share
      FROM participants p
      JOIN games g
        ON g.id = p.game_id
      JOIN teams t
        ON t.game_id = p.game_id
        AND t.team_id = p.team_id
      JOIN teams ot
        ON ot.game_id = p.game_id
        AND ot.team_id != p.team_id
      -- JOIN team_damage td
      --   ON td.game_id = p.game_id
      --   AND td.team_id = p.team_id
      -- WHERE p.role = 'JUNGLE'
      GROUP BY p.name
      HAVING games >= 4
      ORDER BY
        -- dmg_share ASC,
        dp DESC,
        cspm DESC,
        kp DESC,
        gpm  DESC
      LIMIT 180 );
SQL

