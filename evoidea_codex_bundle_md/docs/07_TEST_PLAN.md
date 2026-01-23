# 07_TEST_PLAN — как проверить “работает как задумано”

## 1) Unit tests (без LLM)
- selection:
  - elite_count сохраняется
  - population ограничивается
  - diversity slot добавляет не только топ
- stop conditions:
  - threshold stop
  - stagnation stop
  - max_rounds stop
- data invariants:
  - parents/origin правила
  - status filtering
- serialization:
  - state.json round-trip

## 2) Integration tests (MockLlmProvider)
Сценарий:
- generator -> фиксированные идеи
- critic -> фиксированные score
- merger/mutator/refiner -> фиксы

Проверки:
- после N итераций выбран ожидаемый best_idea_id
- создан final.json по схеме
- history.ndjson имеет события по шагам

## 3) Smoke test (опционально, real codex)
`evoidea run --mode codex --prompt "..."`
Проверка: не падает, создаёт файлы.

## 4) Regression suite
- набор fixture сценариев easy/medium/hard
- проверка, что структура output не ломается
