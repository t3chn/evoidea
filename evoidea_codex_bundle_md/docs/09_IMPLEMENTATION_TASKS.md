# 09_IMPLEMENTATION_TASKS — implementation checklist (for an agent)

## Step 0: Scaffold
- `cargo new evoidea`
- deps: serde, serde_json, anyhow, tracing, tracing-subscriber, clap
- fmt/clippy

## Step 1: Data model
- structs: RunConfig, Idea, Facets, Scores, State, Event, FinalResult
- serde derives
- read/write JSON artifacts

## Step 2: Storage
- FileStorage: init_run/save/load/append_event/save_final
- SqliteStorage (feature flag) — optional

## Step 3: Selection/Scoring
- weighted sum
- select elite + diversity
- stop conditions

## Step 4: LlmProvider
- MockLlmProvider (fixtures)
- CodexExecProvider: spawn `codex exec --output-schema <schema> -o <tmp>`
- CommandProvider: generic exec (optional)

## Step 5: Phases
- GeneratePhase
- CriticPhase
- SelectPhase
- CrossoverPhase
- MutationPhase
- RefinePhase
- FinalPhase

## Step 6: CLI
- run/resume/show/validate/export

## Step 7: Schemas
- create `schemas/*.json` from `non_md/schemas/*.json.md`

## Step 8: Tests
- unit + integration (mock)
- golden fixtures

## Step 9: Skill
- `.codex/skills/evo-ideator/SKILL.md` and references
- materialize `run_evoidea.sh` from `non_md/codex_skill_scripts/*.md`

## Step 10: Evals harness (optional)
- materialize files from `non_md/evals/*.md`
