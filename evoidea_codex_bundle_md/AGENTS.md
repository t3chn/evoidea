# AGENTS.md — instructions for a Codex CLI agent (from-scratch implementation)

## Goal
Implement a local Rust project `evoidea` plus a repo Codex skill `.codex/skills/evo-ideator` that iteratively:
1) generates a population of ideas,
2) scores them with a rubric (multi-criteria),
3) selects the best while preserving diversity,
4) crossovers/mutates,
5) refines the top ideas,
6) stops by rules,
7) returns **exactly 1 best idea** (MODE=IDEATION) and writes run artifacts.

## Context: how to use this bundle
This bundle contains:
- Markdown specs in `docs/` and `.codex/skills/...`
- additional files (schemas/scripts/evals) stored as `.md` under `non_md/` that must be materialized into real files by copying contents and removing the `.md` suffix.

## Mandatory principles
1) Read `docs/00_SPEC.md ... docs/09_IMPLEMENTATION_TASKS.md` first.
2) Work in small iterations. After each iteration run:
   - `cargo fmt`
   - `cargo clippy -- -D warnings`
   - `cargo test`
3) Tests must be deterministic:
   - unit/integration tests should use `MockLlmProvider` by default
   - real `codex exec` calls are smoke/e2e only (optional)
4) Keep dependencies minimal. Any new crate must be justified.
5) Canonical storage is JSON (and/or a SQLite feature flag later).
6) Do not mix modes:
   - MODE=IDEATION: return only the best idea (no detailed implementation plan)
   - MODE=PLANNING: separate command/mode (can be added later)
7) Structured LLM outputs:
   - for Codex CLI use `--output-schema` (see `non_md/schemas/*.json.md`)
   - all object schemas must set `additionalProperties: false` (strict Structured Outputs)

## Definition of Done (DoD)
- `cargo test` passes locally.
- `cargo run -- run --mode mock --prompt "..."` creates `runs/<run_id>/` and writes `final.json`.
- `final.json` contains exactly 1 best idea plus rationale and score.
- Skill `.codex/skills/evo-ideator` exists and can be executed (via a materialized script).
- Docs/specs/test plan are present.

## Code style
- Rust edition 2021.
- Errors: `anyhow` (top-level), `thiserror` (domain errors if needed).
- Logging: `tracing` + `tracing-subscriber`.
- CLI: `clap`.
- Serde: `serde`, `serde_json`.

## Suggested order of work
1) Create a git repository (Codex CLI often requires a git repo for exec mode; use `--skip-git-repo-check` if needed).
2) Implement the Rust CLI per `docs/09_IMPLEMENTATION_TASKS.md`.
3) Materialize the extra files from `non_md/`:
   - `schemas/*.json`
   - `.codex/skills/evo-ideator/scripts/run_evoidea.sh`
   - `evals/*` (if you add the eval harness)
4) Run unit + integration tests.
5) (Optional) smoke run in codex mode:
   - `cargo run -- run --mode codex --prompt "..." --search`
