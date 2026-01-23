# evoidea — MVP Design (IDEATION-only)

Date: 2026-01-23

## Context
This repository currently contains a spec bundle in `evoidea_codex_bundle_md/` (spec/architecture/pipeline/test plan + skill).
The MVP goal is to materialize it into a working Rust CLI `evoidea` and a Codex skill `evo-ideator`.

## Goals / Non-goals
**Goal:** a local, reproducible CLI that takes `--prompt`, runs a memetic loop (generate -> score -> select -> crossover/mutate -> refine -> stop), returns **exactly one** best idea (MODE=IDEATION), and writes run artifacts into `runs/<run_id>/`.

**Non-goals (for MVP):**
- MODE=PLANNING (separate command/mode later).
- SQLite/Turso storage (feature flag later).
- \"Idea quality\" as an eval metric (MVP only validates structure/determinism/invariants).

## Implementation Options (Choice)
1) **Minimal loop (recommended first):** implement pipeline skeleton + FileStorage + MockLlmProvider, with phases enabled via config; start with Generate/Score/Select/Refine/Stop and add Crossover/Mutation later.
Pros: fastest path to green integration tests and stable artifacts.

2) Full loop from day one (all phases).
Pros: fewer follow-up iterations, cons: harder to debug without baseline infra (tests + storage) in place.

For MVP we pick (1): a deterministic vertical slice with artifacts is more important than completeness.

## Architecture (per specs, with MVP simplifications)
- `Orchestrator` owns config/run_id, initializes storage, builds the pipeline, and drives iterations.
- `Pipeline` is a list of `Phase`; each `run(state, ctx) -> state` and writes events to `history.ndjson`.
- `Storage` (MVP): `FileStorage` under `runs/<run_id>/` (config/state/history/final).
- `LlmProvider` (MVP): `MockLlmProvider` (fixtures), interface remains compatible with `CodexExecProvider`.
- `Scoring/Selection`: weighted sum + elite + diversity slot; stop conditions (threshold/stagnation/max_rounds).

## Testing (TDD-first)
- Unit: selection, stop conditions, origin/parents/status invariants, JSON round-trip.
- Integration (MockLlmProvider): 2-4 iterations, validate best_idea_id, artifacts, `final.json` structure.
- Smoke (optional): `--mode codex` does not crash and creates files.

## Quality Tooling
Use prek (`uvx prek ...`) with hooks for:
- repo hygiene (merge conflicts/whitespace/EOF)
- Rust formatting/linting/tests (for `*.rs`).

## Decisions

**Scoring weights (MVP):** all weights = 1.0; risk inverted via `(10 - risk)` before summing.
- Rationale: YAGNI, simpler tests, trivial to add config later.

**Diversity slot (MVP):** random from mid-rank (positions 30%-70% by score).
- Rationale: simplest way to avoid local optima; novelty heuristic can be added later if needed.
