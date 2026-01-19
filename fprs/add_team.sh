sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"
.parameter set :team "$2"

.headers on
.mode column

SELECT
    m.id,
    json_extract(p.value, '$.riotIdGameName') AS name,
    json_extract(p.value, '$.teamId') AS team_id
FROM game m
JOIN json_each(m.data, '$.info.participants') p
WHERE name = :name
SQL

sqlite3 ~/.config/fprs/app.db <<SQL
.parameter init
.parameter set :name "$1"
.parameter set :team "$2"

-- Update team_1 or team_2 based on teamId for matching player name
UPDATE game
SET
    team_1 = CASE 
                 WHEN team_1 IS NULL
                 AND json_extract(p.value, '$.teamId') = 100 THEN :team
                 ELSE team_1
             END,
    team_2 = CASE 
                 WHEN team_2 IS NULL 
                 AND json_extract(p.value, '$.teamId') = 200 THEN :team
                 ELSE team_2
             END
FROM json_each(game.data, '$.info.participants') p
WHERE json_extract(p.value, '$.riotIdGameName') = :name;
SQL

