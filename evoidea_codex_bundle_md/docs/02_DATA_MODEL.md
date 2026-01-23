# 02_DATA_MODEL — structs, invariants, schemas

## 1) Core entities

### RunConfig
- `run_id` (uuid)
- `mode`: mock|codex|command
- `prompt`: string
- `language`: default "ru" in the original spec (repo policy may override)
- `max_rounds`: u32
- `population_size`: u32
- `elite_count`: u32
- `mutation_count`: u32
- `crossover_count`: u32
- `wildcard_count`: u32
- `stagnation_patience`: u32
- `score_threshold`: f32
- `search_enabled`: bool
- `scoring_weights`: weights per criterion

### Idea
- `id`: uuid
- `gen`: u32
- `origin`: generated|crossover|mutated|refined
- `parents`: [uuid]
- `title`: string
- `summary`: string
- `facets` (all strings):
  - `audience`
  - `jtbd`
  - `differentiator`
  - `monetization`
  - `distribution`
  - `risks`
- `scores`: per-criterion scores (0..10)
- `overall_score`: number|null
- `judge_notes`: string|null
- `status`: active|archived

### State
- `run_id`
- `iteration`: u32
- `ideas`: [Idea]
- `best_idea_id`: uuid|null
- `best_score`: number|null
- `stagnation_counter`: u32

### Event (history.ndjson)
- `ts`
- `iteration`
- `type`: generated|scored|selected|crossover|mutated|refined|stopped
- `payload`: object (minimal)

## 2) Invariants
- `Idea.id` is unique.
- `parents` is empty for generated; non-empty for crossover/mutated/refined.
- status=archived ideas do not participate in selection.
- overall_score is computed only after scoring.

## 3) JSON Schema for Structured Outputs (Codex)
Schemas for LLM tasks live in `non_md/schemas/*.json.md`.
After materialization they should exist as real files in `schemas/*.json` and be used with:
`codex exec --output-schema ./schemas/<name>.json ...`

Important:
- for objects always set `additionalProperties: false`.
