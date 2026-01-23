# 05_STORAGE — файлы, SQLite, (опц.) Turso/libSQL

## 1) FileStorage (default)
Структура:
runs/<run_id>/
  config.json
  state.json
  history.ndjson
  final.json
  final.md (optional)

Плюсы:
- минимально
- легко дебажить
- переносимо

## 2) SqliteStorage (feature flag)
Использовать когда:
- много запусков
- нужен поиск по истории
- нужна дедупликация/аналитика

Таблицы (минимум):
- runs(run_id TEXT PK, created_at, config_json)
- ideas(id TEXT PK, run_id, gen, origin, parents_json, title, summary, facets_json, status)
- scores(idea_id, criterion, value)
- events(run_id, ts, iteration, type, payload_json)
- finals(run_id, final_json)

## 3) Turso/libSQL (будущее)
Если когда-нибудь понадобится синхронизация/репликация, можно заменить backend.
Но в MVP не реализуем.
