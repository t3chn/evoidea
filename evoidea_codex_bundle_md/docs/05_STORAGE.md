# 05_STORAGE — files, SQLite, (optional) Turso/libSQL

## 1) FileStorage (default)
Layout:
runs/<run_id>/
  config.json
  state.json
  history.ndjson
  final.json
  final.md (optional)

Pros:
- minimal
- easy to debug
- portable

## 2) SqliteStorage (feature flag)
Use when:
- many runs
- you need history search
- you need dedup/analytics

Tables (minimum):
- runs(run_id TEXT PK, created_at, config_json)
- ideas(id TEXT PK, run_id, gen, origin, parents_json, title, summary, facets_json, status)
- scores(idea_id, criterion, value)
- events(run_id, ts, iteration, type, payload_json)
- finals(run_id, final_json)

## 3) Turso/libSQL (future)
If you ever need sync/replication, you can swap the backend.
Not part of the MVP.
