---
name: evo-ideator
description: Iteratively generate, research, evolve, and select the single best idea for a given prompt (memetic loop with scoring, crossover, mutation, and refinement).
metadata:
  short-description: Memetic idea evolver (best-of loop)
---

## Когда использовать
Используй этот скилл, когда пользователь просит:
- придумать проект / продукт / стратегию
- найти решение задачи
- сгенерировать варианты и выбрать лучший итеративно

## Что делает
Запускает итеративный меметический цикл и возвращает:
- ОДНУ лучшую идею (кратко и практично) + обоснование по score

## Workflow
1) Если в репо есть бинарник `evoidea`:
   - Запусти материализованный скрипт:
     - см. `non_md/codex_skill_scripts/run_evoidea.sh.md`
2) Если бинарника нет:
   - Выполни instruction-only loop:
     - Generate 8–12 ideas
     - Score with rubric
     - Select top + diversity
     - Crossover/mutate
     - Refine top 2–3
     - Stop after 3–6 rounds or earlier if clear winner
   - Сохрани артефакты в `runs/<run_id>/`

## Формат ответа пользователю (IDEATION)
- Название
- 5–10 буллетов: проблема, аудитория, решение, монетизация, дистрибуция, риски, почему выиграло
Без подробного плана реализации, если пользователь не просит.

## Ссылки
- `references/RUNBOOK.md`
- `references/OUTPUT_FORMAT.md`
- `references/TROUBLESHOOTING.md`
