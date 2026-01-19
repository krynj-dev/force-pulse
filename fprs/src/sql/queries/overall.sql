SELECT
  COUNT(*) / 2 AS games,
    SUM(
      CASE 
        WHEN json_extract(t.value, '$.win') = TRUE 
          AND json_extract(t.value, '$.teamId') = 100 
        THEN 1 ELSE 0
      END) AS blue_wins,
    SUM(CASE WHEN json_extract(t.value, '$.win') = true AND json_extract(t.value, '$.teamId') = 200 THEN 1 ELSE 0 END) AS red_wins,
    CAST( 
      ROUND( 
        AVG(json_extract(m.data, '$.info.gameDuration')),
      0)
    AS INTEGER) as game_length_avg,
    MIN(json_extract(m.data, '$.info.gameDuration')) as game_length_min,
    MAX(json_extract(m.data, '$.info.gameDuration')) as game_length_max
FROM game m
JOIN json_each(m.data, '$.info.teams') t;
