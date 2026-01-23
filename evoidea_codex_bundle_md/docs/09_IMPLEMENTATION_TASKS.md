# 09_IMPLEMENTATION_TASKS — пошаговый план реализации (для агента)

## Шаг 0: Scaffold
- `cargo new evoidea`
- deps: serde, serde_json, anyhow, tracing, tracing-subscriber, clap
- fmt/clippy

## Шаг 1: Data model
- structs: RunConfig, Idea, Facets, Scores, State, Event, FinalResult
- serde derives
- чтение/запись JSON артефактов

## Шаг 2: Storage
- FileStorage: init_run/save/load/append_event/save_final
- SqliteStorage (feature flag) — по желанию

## Шаг 3: Selection/Scoring
- weighted sum
- select elite + diversity
- stop conditions

## Шаг 4: LlmProvider
- MockLlmProvider (fixtures)
- CodexExecProvider: spawn `codex exec --output-schema <schema> -o <tmp>`
- CommandProvider: generic exec (опционально)

## Шаг 5: Phases
- GeneratePhase
- CriticPhase
- SelectPhase
- CrossoverPhase
- MutationPhase
- RefinePhase
- FinalPhase

## Шаг 6: CLI
- run/resume/show/validate/export

## Шаг 7: Schemas
- создать `schemas/*.json` по `non_md/schemas/*.json.md`

## Шаг 8: Tests
- unit + integration (mock)
- golden fixtures

## Шаг 9: Skill
- `.codex/skills/evo-ideator/SKILL.md` и references
- материализовать `run_evoidea.sh` из `non_md/codex_skill_scripts/*.md`

## Шаг 10: Evals harness (optional)
- материализовать файлы из `non_md/evals/*.md`
