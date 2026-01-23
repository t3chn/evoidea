# 01_ARCHITECTURE — модули и расширяемость

## 1. Схема
[CLI] -> [Orchestrator] -> [Pipeline]
                      -> [Storage]
                      -> [LlmProvider]
                      -> [Scoring/Selection]

## 2. Ключевые абстракции

### Phase (плагин)
Каждая фаза реализует:
- `name() -> &str`
- `run(state, ctx) -> state`

Pipeline — список фаз из config.

### LlmProvider
Трейт:
- `generate_json(task: LlmTask, schema_path: &Path) -> serde_json::Value`

Реализации:
- `MockLlmProvider` (fixtures)
- `CodexExecProvider` (spawn `codex exec ... --output-schema <schema.json>`)
- `CommandProvider` (любой внешний CLI — при необходимости)

### Storage
Трейт:
- `init_run(config) -> run_id`
- `load_state / save_state`
- `append_event`
- `save_final`

Реализации:
- `FileStorage` (default)
- `SqliteStorage` (feature flag)

## 3. Расширение “без переписывания”
Добавить фазу = новый модуль + регистрация в конфиге pipeline.
Добавить провайдер = реализация LlmProvider.
Добавить хранилище = реализация Storage.

## 4. Контекст и сокращение
В LLM отправляем компактный контекст:
- только top-K идей + новые
- только нужные поля
- (опционально) TOON для uniform arrays

Канон хранится в JSON.

## 5. Логи и история
- `history.ndjson` как append-only событийный лог по итерациям
- `tracing` span на run_id и iteration
