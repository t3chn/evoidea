---
name: evoidea
description: Evolve startup/product ideas using a memetic algorithm with scoring, selection, and refinement
allowed-tools: ["Read", "Write", "Bash", "Task", "Glob"]
---

# Evoidea - Memetic Idea Evolution

You are running an evolutionary algorithm to generate and refine startup/product ideas.

## Arguments

Parse from `$ARGUMENTS`:
- **prompt**: The main direction for idea generation (required, positional)
- **--rounds N**: Max iterations (default: 3)
- **--population N**: Ideas in first generation (default: 6)
- **--elite N**: Top ideas to refine (default: 2)
- **--threshold N**: Score for early stop (default: 9.0)
- **--resume ID**: Run ID to continue from

Example: `/evoidea "Build tools for solo developers" --rounds 3 --population 6`

## Initialization

1. Parse arguments from `$ARGUMENTS`
2. If `--resume ID` provided:
   - Read `runs/<ID>/state.json`
   - Continue from saved iteration
3. Otherwise:
   - Generate `run_id` using timestamp format: `run-YYYYMMDD-HHMMSS`
   - Create directory `runs/<run_id>/`
   - Initialize config.json and state.json

### config.json format
```json
{
  "run_id": "run-20260123-143022",
  "prompt": "...",
  "max_rounds": 3,
  "population_size": 6,
  "elite_count": 2,
  "score_threshold": 9.0,
  "stagnation_patience": 3
}
```

### Initial state.json format
```json
{
  "run_id": "run-20260123-143022",
  "iteration": 0,
  "ideas": [],
  "best_idea_id": null,
  "best_score": null,
  "stagnation_counter": 0
}
```

## Evolution Loop

For each iteration (1 to max_rounds):

### Phase 1: GENERATE (iteration 1 only)

Use Task tool with `subagent_type: "general-purpose"`:

```
Generate {population_size} startup/product ideas for: "{prompt}"

For each idea, output valid JSON array with objects containing:
- id: unique identifier (e.g., "idea-001")
- title: 5-10 words
- summary: 2-3 sentences describing the concept
- facets:
  - audience: target user segment
  - jtbd: job to be done / problem solved
  - differentiator: what makes it unique
  - monetization: how it makes money
  - distribution: how users find it
  - risks: main challenges

Be diverse. Avoid similar ideas. Output ONLY the JSON array.
```

Parse response and add ideas to state.ideas with:
- `status: "active"`
- `origin: "generated"`
- `gen: 1`
- `parents: []`

### Phase 2: CRITIC

Use Task tool with `subagent_type: "general-purpose"`:

```
Rate each idea 0-10 on these criteria. Be strict and realistic.

CRITERIA (apply in order):
1. feasibility - can a solo dev build MVP in weeks?
2. speed_to_value - time to first paying customer
3. differentiation - unique vs existing solutions
4. market_size - potential revenue scale
5. distribution - organic growth channels available
6. moats - defensibility over time
7. risk - inverse of failure probability (high=less risky)
8. clarity - how well-defined is the concept

IDEAS TO EVALUATE:
{JSON array of active ideas with id, title, summary, facets}

Output JSON object mapping idea IDs to scores:
{
  "idea-001": {
    "feasibility": 7,
    "speed_to_value": 8,
    ...
  },
  ...
}
```

For each idea, compute `overall_score` as average of all criteria.

### Phase 3: SELECT (no LLM)

1. Sort active ideas by overall_score descending
2. Mark bottom half as `status: "archived"`
3. Keep top `elite_count` ideas as `status: "active"`

### Phase 4: REFINE (parallel)

For each active idea, launch parallel Task tool with `subagent_type: "general-purpose"`:

```
Improve this idea based on its critique. Focus on the weakest scores.

IDEA:
{idea JSON with title, summary, facets}

SCORES (0-10):
{scores object}

WEAKEST AREAS: {list 2-3 lowest scoring criteria}

BENCHMARK - Current best idea scores {best_score}/10

Return an improved version with the same JSON structure:
- id: "{new-id}" (e.g., "idea-001-r2" for round 2 refinement)
- title, summary, facets (updated to address weaknesses)
- changes: ["what you changed and why", ...]

Output ONLY valid JSON.
```

Add refined ideas to state.ideas with:
- `origin: "refined"`
- `gen: current_iteration`
- `parents: [original_idea_id]`

Archive the original ideas that were refined.

### Phase 5: COMPRESS

After each iteration:
1. Remove archived ideas from active consideration
2. Keep only essential fields in state
3. Log iteration summary to history.ndjson:
   ```json
   {"ts": "...", "iteration": N, "type": "iteration_complete", "best_score": X, "active_count": Y}
   ```

## Stop Conditions

Check after each iteration:

1. **Threshold reached**: `best_score >= threshold`
   - Stop with reason: "Threshold {threshold} reached with score {best_score}"

2. **Stagnation**: `stagnation_counter >= patience`
   - If new best_score <= previous best_score, increment stagnation_counter
   - If stagnation_counter >= 3, stop with reason: "Stagnation detected"

3. **Max rounds**: `iteration >= max_rounds`
   - Stop with reason: "Completed {max_rounds} rounds"

## Finalization

When stopped:

1. Create `runs/<run_id>/final.json`:
```json
{
  "run_id": "...",
  "prompt": "...",
  "iterations_completed": N,
  "stop_reason": "...",
  "best_idea": { full idea object },
  "runner_up": { second best idea or null }
}
```

2. Present the winning idea to the user in this format:

---
## 🏆 Best Idea: {title}

**Score:** {overall_score}/10

**Audience:** {audience}
**Problem:** {jtbd}
**Solution:** {summary}
**Unique:** {differentiator}
**Monetization:** {monetization}
**Distribution:** {distribution}
**Risks:** {risks}

### Why it won
- {reasons based on highest scores}

---

## Error Handling

- **Invalid JSON from subagent**: Retry once with explicit JSON formatting instructions
- **Resume with non-existent run-id**: Error message and exit
- **All ideas archived**: Re-generate with fresh prompt
- **Write fails**: Save current state and stop gracefully

## State Persistence

After EVERY phase:
1. Write updated state.json
2. Append to history.ndjson

This enables resume at any point.
