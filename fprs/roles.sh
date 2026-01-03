sqlite3 ~/.config/fprs/app.db <<'SQL'
.headers on
.mode column

SELECT
    json_extract(p.value, '$.teamPosition') as role,
        MIN(CAST( json_extract(p.value, '$.kills') AS INTEGER)) as min_k,
        MIN(CAST( json_extract(p.value, '$.deaths') AS INTEGER)) as min_d,
        MIN(CAST( json_extract(p.value, '$.assists')  AS INTEGER)) as min_a,
        ROUND(AVG(json_extract(p.value, '$.kills')), 1) as avg_k,
        ROUND(AVG(json_extract(p.value, '$.deaths')), 1) as avg_d,
        ROUND(AVG(json_extract(p.value, '$.assists')), 1) as avg_a,
        MAX(CAST( json_extract(p.value, '$.kills') AS INTEGER)) as max_k,
        MAX(CAST( json_extract(p.value, '$.deaths') AS INTEGER)) as max_d,
        MAX(CAST( json_extract(p.value, '$.assists') AS INTEGER)) as max_a,
        MIN(CAST( json_extract(p.value, '$.totalDamageDealtToChampions') AS INTEGER)) as min_dmg,
        ROUND(AVG(CAST( json_extract(p.value, '$.totalDamageDealtToChampions') AS INTEGER)), 0) as avg_dmg,
        MAX(CAST( json_extract(p.value, '$.totalDamageDealtToChampions') AS INTEGER)) as max_dmg
  FROM game m
JOIN json_each(m.data, '$.info.participants') p
GROUP BY role;
SQL
