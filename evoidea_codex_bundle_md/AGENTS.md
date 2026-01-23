# AGENTS.md — инструкции для Codex CLI-агента (реализация с нуля)

## Цель
Реализовать локальный Rust-проект `evoidea` + репо-скилл Codex `.codex/skills/evo-ideator`,
который итеративно:
1) генерирует популяцию идей,
2) оценивает по рубрике (мультикритерий),
3) отбирает лучших + сохраняет разнообразие,
4) скрещивает/мутирует,
5) улучшает топ,
6) останавливается по правилам,
7) возвращает **1 лучшую идею** (MODE=IDEATION) и пишет артефакты запуска.

## Контекст: как работать с этим пакетом
Этот zip содержит:
- реальные Markdown-спеки в `docs/` и `.codex/skills/...`
- дополнительные файлы (schemas/scripts/evals) **в формате .md** внутри `non_md/`.
  Их надо материализовать (создать настоящие .json/.sh/.csv/.mjs) копированием содержимого и
  удалением суффикса `.md`.

## Обязательные принципы
1) Сначала прочитай `docs/00_SPEC.md ... docs/09_IMPLEMENTATION_TASKS.md`.
2) Делай маленькие итерации. После каждой:
   - `cargo fmt`
   - `cargo clippy -- -D warnings`
   - `cargo test`
3) Тесты должны быть детерминированными:
   - по умолчанию unit/integration используют `MockLlmProvider`
   - реальные вызовы `codex exec` — только smoke/e2e (опционально)
4) Минимальные зависимости. Любой новый crate — обосновать.
5) Канонический storage: JSON (и/или SQLite feature flag).
   TOON — опционально только для контекстного сжатия (можно сделать позднее).
6) Не смешивать режимы:
   - MODE=IDEATION: **только лучшая идея** (без подробного плана реализации)
   - MODE=PLANNING: отдельная команда/режим (если потребуется позже)
7) Структурированные выходы LLM:
   - для Codex CLI использовать `--output-schema` (см. `non_md/schemas/*.json.md`)
   - во всех схемах `additionalProperties: false` для объектов (строгое правило Structured Outputs)

## Definition of Done (DoD)
- `cargo test` проходит локально.
- `cargo run -- run --mode mock --prompt "..."` создаёт `runs/<run_id>/` и финальный `final.json`.
- В `final.json` ровно 1 лучшая идея + обоснования + score.
- Skill `.codex/skills/evo-ideator` существует и запускается (через материализацию скрипта).
- Документация/спеки/план тестов присутствуют.

## Стиль кода
- Rust edition 2021.
- Ошибки: `anyhow` (верхний уровень), `thiserror` (доменные ошибки по необходимости).
- Логи: `tracing` + `tracing-subscriber`.
- CLI: `clap`.
- Серде: `serde`, `serde_json`.

## Порядок действий (рекомендуемый)
1) Создай git-репозиторий (Codex CLI обычно требует git repo для exec-режима; если нужно — используй `--skip-git-repo-check`).
2) Реализуй Rust CLI согласно `docs/09_IMPLEMENTATION_TASKS.md`.
3) Материализуй дополнительные файлы из `non_md/`:
   - `schemas/*.json`
   - `.codex/skills/evo-ideator/scripts/run_evoidea.sh`
   - `evals/*` (если делаешь eval harness)
4) Прогони unit+integration тесты.
5) (Опционально) smoke run в режиме codex:
   - `cargo run -- run --mode codex --prompt "..." --search`
