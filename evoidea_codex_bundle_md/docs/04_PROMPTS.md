# 04_PROMPTS — шаблоны LLM-задач

## Общие правила
- LLM возвращает **только JSON** по схеме, без текста вокруг.
- Язык контента: config.language (по умолчанию ru).
- Если данных недостаточно — делай разумные предположения и указывай их в judge_notes.

## Generator
Вход: prompt + constraints + (опц) research_bullets + (опц) current_best
Выход: объект { ideas: [IdeaDraft...] } по schema `generator.output.schema.json`.

## Critic/Judge
Вход: идеи (id+контент) + rubric + weights + constraints
Выход: объект { patches: [IdeaScorePatch...] } по schema `critic.output.schema.json`.

## Merger (Crossover)
Вход: ideaA + ideaB
Выход: объект { idea: IdeaDraft } по schema `merger.output.schema.json`.

## Mutator
Вход: исходная идея + mutation_type (audience|monetization|distribution|differentiator|jtbd)
Выход: объект { idea: IdeaDraft } по schema `mutator.output.schema.json`.

## Refiner
Вход: идея + judge_notes
Выход: объект { patch: IdeaRefinePatch } по schema `refiner.output.schema.json`.

## Final composer
Вход: top ideas (2–5)
Выход: `final.output.schema.json`.
