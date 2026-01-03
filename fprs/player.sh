echo 'Overall'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.riotIdGameName') AS name,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent,
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
      AVG(json_extract(p.value, '$.totalDamageDealtToChampions')) as damage,
        printf('%d:%02d', AVG(json_extract(m.data, '$.info.gameDuration')) / 60, AVG(json_extract(m.data, '$.info.gameDuration')) % 60) AS game_length
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name
ORDER BY win_percent
SQL

echo ''
echo 'By role'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.riotIdGameName') AS name,
    json_extract(p.value, '$.teamPosition') AS role,
    COUNT(*) AS games,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent,
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
      SUM(json_extract(p.value, '$.goldEarned')) /
      ( SUM(json_extract(m.data, '$.info.gameDuration')) / 60)
      AS gpm,
      SUM(json_extract(p.value, '$.totalDamageDealtToChampions')) as damage
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name
GROUP BY role
ORDER BY games DESC, win_percent DESC
SQL

echo ''
echo 'Champions'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.riotIdGameName') AS name,
    json_extract(p.value, '$.championName') AS champion,
    COUNT(*) as games,
    SUM(CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) AS wins,
    SUM(CASE WHEN json_extract(p.value, '$.win') = false THEN 1 ELSE 0 END) AS losses,
    SUM(100 * CASE WHEN json_extract(p.value, '$.win') = true THEN 1 ELSE 0 END) / (  COUNT(*) ) as win_percent,
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
          SUM(json_extract(p.value, '$.goldEarned')) /
      ( SUM(json_extract(m.data, '$.info.gameDuration')) / 60)
      AS gpm,
      SUM(json_extract(p.value, '$.totalDamageDealtToChampions')) as damage

FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name
GROUP BY champion
ORDER BY games DESC, win_percent DESC
SQL

echo ''
echo 'Match history'
sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"

.headers on
.mode column

SELECT
    json_extract(p.value, '$.riotIdGameName') AS player,
    json_extract(p.value, '$.championName') AS champion,
    json_extract(p.value, '$.teamPosition') AS role,
    CASE
        WHEN json_extract(p.value, '$.win') = 1 THEN 'WIN'
        ELSE 'LOSS'
    END AS result,
    json_extract(p.value, '$.kills')     AS kills,
    json_extract(p.value, '$.deaths')    AS deaths,
    json_extract(p.value, '$.assists')   AS assists,
    ROUND(
        (json_extract(p.value, '$.kills') +
         json_extract(p.value, '$.assists')) * 1.0 /
        CASE
            WHEN json_extract(p.value, '$.deaths') = 0 THEN 1
            ELSE json_extract(p.value, '$.deaths')
        END,
        2
    ) AS kda,
          json_extract(p.value, '$.goldEarned') /
      ( json_extract(m.data, '$.info.gameDuration') / 60)
      AS gpm,

      json_extract(p.value, '$.totalDamageDealtToChampions') as damage,
      json_extract(p.value, '$.totalDamageDealtToChampions') / (json_extract(m.data, '$.info.gameDuration') / 60) as dpm,
        printf('%d:%02d', json_extract(m.data, '$.info.gameDuration') / 60, json_extract(m.data, '$.info.gameDuration') % 60) AS game_length
    -- json_extract(m.data, '$.info.gameEndTimestamp') as date_
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name
ORDER BY json_extract(m.data, '$.info.gameEndTimestamp') DESC
LIMIT 50;
SQL

