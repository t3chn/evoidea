# evoidea — MVP дизайн (IDEATION-only)

Дата: 2026-01-23

## Контекст
В репозитории лежит пакет спецификаций `evoidea_codex_bundle_md/` (spec/архитектура/pipeline/test plan + skill).
Цель MVP — материализовать это в рабочий Rust CLI `evoidea` и Codex skill `evo-ideator`.

## Цель / не-цели
**Цель:** локальный, воспроизводимый CLI, который по `--prompt` делает меметический цикл (generate → score → select → crossover/mutate → refine → stop) и возвращает **ровно 1** лучшую идею (MODE=IDEATION), сохраняя артефакты запуска в `runs/<run_id>/`.

**Не-цели (для MVP):**
- MODE=PLANNING (отдельная команда/режим позже).
- SQLite/Turso storage (feature-флаг позже).
- “Качество идеи” как eval-метрика (в MVP проверяем структуру/детерминизм/инварианты).

## Варианты реализации (выбор)
1) **Минимальный цикл (рекомендовано для старта):** реализовать каркас pipeline + FileStorage + MockLlmProvider, но включать фазы по конфигу; начать с Generate/Score/Select/Refine/Stop и затем добавлять Crossover/Mutation.
Плюсы: быстрее выйти на зелёные интеграционные тесты и артефакты запуска.

2) Полный цикл сразу (все фазы).
Плюсы: меньше “переделок” после MVP, минусы: сложнее отлаживать без базовой инфраструктуры тестов/стора.

Для MVP берём (1): ранняя “вертикаль” с детерминизмом и артефактами важнее полноты.

## Архитектура (как в спеках, с MVP-упрощениями)
- `Orchestrator` держит config/run_id, инициирует storage, собирает pipeline, крутит итерации.
- `Pipeline` — список `Phase`, каждая `run(state, ctx) -> state` и пишет события в `history.ndjson`.
- `Storage` (MVP): `FileStorage` под `runs/<run_id>/` (config/state/history/final).
- `LlmProvider` (MVP): `MockLlmProvider` (fixtures), интерфейс заранее совместим с `CodexExecProvider`.
- `Scoring/Selection`: weighted sum + elite + diversity slot; stop conditions (threshold/stagnation/max_rounds).

## Тестирование (TDD-first)
- Unit: selection, stop conditions, инварианты origin/parents/status, JSON round-trip.
- Integration (MockLlmProvider): прогон 2–4 итераций, проверка best_idea_id, артефактов, `final.json` по схеме.
- Smoke (опционально): `--mode codex` без падений и с созданием файлов.

## Инструменты качества
Включаем prek (`uvx prek ...`) с хуками для:
- базовой гигиены репо (конфликты/whitespace/конечные переводы строк)
- Rust форматирования/линтинга/тестов (по файлам `*.rs`).

## Открытые решения (можно закодировать как отдельные задачи)
- Конкретные веса scoring (по умолчанию = 1.0, кроме risk = -1.0?).
- Алгоритм diversity slot (рандом из середины ранга vs простая novelty-эвристика).
