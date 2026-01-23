# 07_TEST_PLAN — how to verify behavior

## 1) Unit tests (no LLM)
- selection:
  - elite_count preserved
  - population bounded
  - diversity slot is not top-only
- stop conditions:
  - threshold stop
  - stagnation stop
  - max_rounds stop
- data invariants:
  - parents/origin rules
  - status filtering
- serialization:
  - state.json round-trip

## 2) Integration tests (MockLlmProvider)
Scenario:
- generator -> fixed ideas
- critic -> fixed scores
- merger/mutator/refiner -> fixed outputs

Checks:
- after N iterations, best_idea_id is the expected one
- final.json is created and matches the expected structure
- history.ndjson contains step events

## 3) Smoke test (optional, real codex)
`evoidea run --mode codex --prompt "..."`
Check: does not crash and creates files.

## 4) Regression suite
- a set of fixture scenarios: easy/medium/hard
- verify that output structure does not regress
