sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

WITH
teams AS (
    SELECT
        CASE
            WHEN json_extract(p.value, '$.teamId') = 100 THEN m.team_1
            WHEN json_extract(p.value, '$.teamId') = 200 THEN m.team_2
        END AS team_name,
        substr(CASE
            WHEN json_extract(p.value, '$.teamId') = 100 THEN m.team_1
            WHEN json_extract(p.value, '$.teamId') = 200 THEN m.team_2
        END, 1, 15) AS team_name_trunc,
        SUM(json_extract(p.value, '$.kills')) AS total_kills,
        SUM(json_extract(p.value, '$.deaths')) AS total_deaths,
        SUM(json_extract(p.value, '$.assists')) AS total_assists,
        ROUND(SUM(json_extract(p.value, '$.kills') + json_extract(p.value, '$.assists')) * 1.0 /
              MAX(1, SUM(json_extract(p.value, '$.deaths'))), 2) AS team_kda,
        SUM(CASE WHEN json_extract(p.value, '$.win') = 1 THEN 1 ELSE 0 END) / 5 AS wins,
        SUM(CASE WHEN json_extract(p.value, '$.win') = 0 THEN 1 ELSE 0 END) / 5 AS losses,
        (100 * SUM(CASE WHEN json_extract(p.value, '$.win') = 1 THEN 1 ELSE 0 END) / COUNT(*)) AS wr
    FROM game m
    JOIN json_each(m.data, '$.info.participants') p
    GROUP BY team_name
),

stats AS (
    SELECT
        g.id AS game_id,
        COUNT(*) AS games,
        CASE
            WHEN json_extract(t.value, '$.teamId') = 100 THEN g.team_1
            WHEN json_extract(t.value, '$.teamId') = 200 THEN g.team_2
        END AS team_name,
        json_extract(t.value, '$.teamId') AS team_id,
        json_extract(t.value, '$.objectives.champion.kills') AS ckill,
        SUM( CASE WHEN json_extract(t.value, '$.objectives.champion.first') = TRUE THEN 1 ELSE 0 END) AS fb,
        SUM( CASE WHEN json_extract(t.value, '$.objectives.tower.first') = TRUE THEN 1 ELSE 0 END ) AS ft,
        SUM( CASE WHEN json_extract(t.value, '$.objectives.dragon.first') = TRUE THEN 1 ELSE 0 END ) AS fd,
        SUM( CASE WHEN json_extract(t.value, '$.objectives.baron.first') = TRUE THEN 1 ELSE 0 END ) AS fn,
        SUM( CASE WHEN json_extract(t.value, '$.objectives.riftHerald.first') = TRUE THEN 1 ELSE 0 END ) AS fh,
        SUM( CASE WHEN json_extract(t.value, '$.objectives.atakhan.first') = TRUE THEN 1 ELSE 0 END ) AS fat,
        SUM( CASE WHEN json_extract(t.value, '$.objectives.atakhan.first') IS NOT NULL THEN 1 ELSE 0 END ) AS fat_p,
        AVG( json_extract(t.value, '$.objectives.dragon.kills') ) AS td,
        AVG( json_extract(t.value, '$.objectives.horde.kills') ) AS grubs
    FROM game g
    JOIN json_each(g.data, '$.info.teams') t
    WHERE json_extract(t.value, '$.objectives.tower.first') IS NOT NULL
    GROUP BY team_name
)

SELECT
  ROW_NUMBER() OVER (ORDER BY 
    wr DESC    ) AS no,
    teams.team_name_trunc, teams.team_kda, teams.wins, teams.losses, teams.wr,
    100 * stats.fb / stats.games AS fbp,
    100 * stats.ft / stats.games AS ftp,
    100 * stats.fd / stats.games AS fdp,
    100 * stats.fn / stats.games AS fnp,
    100 * stats.fh / stats.games AS fhp,
    100 * stats.fat / stats.fat_p AS fap,
    ROUND( stats.grubs , 1) AS grubs
    FROM teams JOIN stats ON teams.team_name=stats.team_name
ORDER BY teams.team_kda DESC;

SQL
