# 02_DATA_MODEL — структуры, инварианты, схемы

## 1) Основные сущности

### RunConfig
- `run_id` (uuid)
- `mode`: mock|codex|command
- `prompt`: string
- `language`: "ru" default
- `max_rounds`: u32
- `population_size`: u32
- `elite_count`: u32
- `mutation_count`: u32
- `crossover_count`: u32
- `wildcard_count`: u32
- `stagnation_patience`: u32
- `score_threshold`: f32
- `search_enabled`: bool
- `scoring_weights`: объект весов по критериям

### Idea
- `id`: uuid
- `gen`: u32
- `origin`: generated|crossover|mutated|refined
- `parents`: [uuid]
- `title`: string
- `summary`: string
- `facets` (все строки):
  - `audience`
  - `jtbd`
  - `differentiator`
  - `monetization`
  - `distribution`
  - `risks`
- `scores`: объект критериев (0..10)
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
- `payload`: object (минимум)

## 2) Инварианты
- `Idea.id` уникален.
- `parents` пуст для generated; не пуст для crossover/mutated/refined.
- status=archived идеи не участвуют в отборе.
- overall_score вычисляется только после scoring.

## 3) JSON Schema для Structured Outputs (Codex)
Схемы для LLM задач лежат в `non_md/schemas/*.json.md`.
После материализации они должны оказаться в `schemas/*.json` и использоваться с:
`codex exec --output-schema ./schemas/<name>.json ...`

Важно:
- для объектов всегда ставить `additionalProperties: false`.
