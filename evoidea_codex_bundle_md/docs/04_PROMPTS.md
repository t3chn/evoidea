# 04_PROMPTS — LLM task templates

## General rules
- The LLM returns **JSON only** per schema, with no surrounding text.
- Content language: config.language (default: ru in the original spec; repo policy may override).
- If data is insufficient, make reasonable assumptions and record them in judge_notes.

## Generator
Input: prompt + constraints + (optional) research_bullets + (optional) current_best
Output: object `{ ideas: [IdeaDraft...] }` per schema `generator.output.schema.json`.

## Critic/Judge
Input: ideas (id + content) + rubric + weights + constraints
Output: object `{ patches: [IdeaScorePatch...] }` per schema `critic.output.schema.json`.

## Merger (Crossover)
Input: ideaA + ideaB
Output: object `{ idea: IdeaDraft }` per schema `merger.output.schema.json`.

## Mutator
Input: source idea + mutation_type (audience|monetization|distribution|differentiator|jtbd)
Output: object `{ idea: IdeaDraft }` per schema `mutator.output.schema.json`.

## Refiner
Input: idea + judge_notes
Output: object `{ patch: IdeaRefinePatch }` per schema `refiner.output.schema.json`.

## Final composer
Input: top ideas (2-5)
Output: `final.output.schema.json`.
