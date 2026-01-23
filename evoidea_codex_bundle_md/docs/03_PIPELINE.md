# 03_PIPELINE — step-by-step algorithm

## Iteration i

### Step 0: PrepareContext
- gather top-K active ideas
- assemble rubric and constraints

### Step 1: Research (optional)
If `search_enabled` is on, run a research task:
- collect 5-15 facts/links/observations about the topic
- store them in `state` (e.g. `state.notes.research`) or in a separate file

### Step 2: Generate
- create N new ideas (up to population_size)
- fill facets for each idea

### Step 3: Critic/Score
- score ideas on criteria 0..10
- return judge_notes
- compute overall_score as a weighted sum (in code)

Default criteria:
- feasibility
- speed_to_value
- differentiation
- market_size
- distribution
- moats
- risk (inverted: higher risk => lower contribution)
- clarity

### Step 4: Select
- elite: top elite_count by overall_score
- diversity slots:
  - random from mid-rank (or a simplified novelty heuristic)
- trim to population_size

### Step 5: Crossover
- pick pairs from top-M
- merger creates 1 new idea per pair
- add crossover_count ideas

### Step 6: Mutation
- pick ideas (top + random)
- change exactly 1 facet/aspect
- add mutation_count ideas

### Step 7: Refine
- take top-K
- improve based on judge_notes (more specific, mitigate risks)

### Step 8: Stop conditions
- best_score >= threshold
- stagnation_counter >= patience
- iteration >= max_rounds
