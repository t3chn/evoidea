# 00_SPEC — requirements and boundaries

## 1. What we are building
A Rust CLI tool plus a Codex skill. The tool implements a \"memetic\" loop:
generate -> score -> select -> crossover -> mutate -> refine -> repeat.

## 2. Input
- `prompt` (string): user task (project idea, problem solution, strategy, etc.)
- Optional constraints:
  - niche/market
  - time-to-MVP
  - revenue target (e.g. $10k/mo)
  - stack/resource constraints
  - output language (default: ru in the original spec; repository policy may override)

## 3. Output (MODE=IDEATION)
One best idea:
- `title`
- `summary` (1-3 paragraphs)
- `facets`:
  - `audience`
  - `jtbd`
  - `differentiator`
  - `monetization`
  - `distribution`
  - `risks`
- `scores` per criterion + `overall_score`
- `why_won`: 2-6 reasons

Important: IDEATION must **not** produce a detailed implementation plan unless the user asked for it.

## 4. Output (MODE=PLANNING)
A separate command/mode (can be added later):
- implementation plan for the chosen idea (phases, tasks, metrics, GTM)

## 5. Run artifacts
Each run creates `runs/<run_id>/`:
- `config.json` — applied parameters
- `state.json` — current population state
- `history.ndjson` — per-iteration events (scoring/selection/mutations)
- `final.json` — final result
- `final.md` — (optional) human-friendly markdown report

## 6. Algorithm (high level)
Iteration i:
1) (optional) research: collect facts/links about the niche
2) generate: add ideas
3) critic: score ideas (multi-criteria)
4) select: elite + diversity
5) crossover: combine top ideas
6) mutate: create variations
7) refine: improve top-K
8) stop: if a stop condition is met

## 7. Stop conditions
- `best_score >= threshold`
- or no improvement for N iterations (`stagnation_patience`)
- or reached `max_rounds`

## 8. Non-functional requirements
- Local-first.
- Testable and reproducible (via a mock provider).
- Minimal dependencies.
- Extensible: new phases can be added without rewriting the core.

## 9. Definition of Done
- `cargo test` is green
- a mock run writes artifacts and a valid `final.json`
- the skill exists
