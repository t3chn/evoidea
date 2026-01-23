# 03_PIPELINE — точный алгоритм по шагам

## Итерация i

### Шаг 0: PrepareContext
- собрать top-K идей (active)
- сформировать rubric и constraints

### Шаг 1: Research (optional)
Если включен `search_enabled`, вызвать research-задачу:
- собрать 5–15 фактов/ссылок/наблюдений по теме
- сохранить в `state` (например, `state.notes.research` или отдельный файл)

### Шаг 2: Generate
- создать N новых идей (до population_size)
- у каждой идеи заполнить facets

### Шаг 3: Critic/Score
- оценить идеи по критериям 0..10
- вернуть judge_notes
- вычислить overall_score как взвешенную сумму (в коде)

Критерии по умолчанию:
- feasibility
- speed_to_value
- differentiation
- market_size
- distribution
- moats
- risk (инвертируется: выше риск => ниже вклад)
- clarity

### Шаг 4: Select
- элита: top elite_count по overall_score
- diversity slots:
  - random из середины ранга (или упрощенная novelty по токенам)
- обрезать до population_size

### Шаг 5: Crossover
- выбрать пары из top-M
- мерджер делает 1 новую идею на пару
- добавить crossover_count идей

### Шаг 6: Mutation
- выбрать идеи (top + random)
- изменить ровно 1 аспект
- добавить mutation_count идей

### Шаг 7: Refine
- взять top-K
- улучшить по judge_notes (конкретика, устранение рисков)

### Шаг 8: Stop conditions
- best_score >= threshold
- stagnation_counter >= patience
- iteration >= max_rounds
