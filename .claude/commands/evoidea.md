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
- **--profile FILE**: Path to a preference profile JSON exported by `evoidea profile export`

### Constraint Arguments (optional)
- **--budget N**: Max budget in USD (e.g., `--budget 500`)
- **--timeline N**: Max timeline in weeks (e.g., `--timeline 4`)
- **--skills LIST**: Required skills, comma-separated (e.g., `--skills rust,python`)
- **--must LIST**: Required elements, comma-separated (e.g., `--must api,saas`)
- **--no LIST**: Forbidden elements, comma-separated (e.g., `--no crypto,hardware,marketplace`)
- **--solo**: Ideas must be buildable by solo developer (flag)

### Domain Expertise (optional)
- **--examples FILE**: Path to JSON file with gold-standard example ideas for few-shot learning
  - Bundled examples: `examples/devtools.json`, `examples/saas.json`, `examples/consumer.json`
  - Examples guide GENERATE phase style and CRITIC phase calibration

Example: `/evoidea "Build tools for solo developers" --rounds 3 --population 6`

Example with constraints:
`/evoidea "Developer tools" --budget 1000 --timeline 4 --skills rust --no crypto,hardware --solo`

Example with domain expertise:
`/evoidea "Developer productivity tools" --examples examples/devtools.json`

Example with a preference profile:
`/evoidea "Developer tools" --profile prefs.json`

## Initialization

1. Parse arguments from `$ARGUMENTS`
2. If `--profile FILE` provided:
   - Read and parse the JSON file
   - Extract weights from `derived.criterion_weights` (preferred) or `criterion_weights`
   - Validate all 8 keys exist: `feasibility`, `speed_to_value`, `differentiation`, `market_size`, `distribution`, `moats`, `risk`, `clarity`
   - Normalize weights so they sum to `1.0`
   - If the profile is missing/invalid or weights cannot be extracted/validated, warn and fall back to uniform weights (all `1/8`)
3. If `--resume ID` provided:
   - Read `runs/<ID>/state.json`
   - If `runs/<ID>/config.json` already contains `scoring_weights`, prefer those (resume should be consistent)
   - If `--profile FILE` is also provided, override `scoring_weights` and rewrite `config.json`
   - Continue from saved iteration
4. Otherwise:
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
  "stagnation_patience": 3,
  "constraints": {
    "budget_usd": 1000,
    "timeline_weeks": 4,
    "required_skills": ["rust"],
    "must_include": ["api"],
    "forbidden": ["crypto", "hardware"],
    "solo_dev": true
  },
  "examples_file": "examples/devtools.json",
  "profile_file": "prefs.json",
  "scoring_weights": {
    "feasibility": 0.125,
    "speed_to_value": 0.125,
    "differentiation": 0.125,
    "market_size": 0.125,
    "distribution": 0.125,
    "moats": 0.125,
    "risk": 0.125,
    "clarity": 0.125
  }
}
```

Note: `constraints` object is optional. Omit or set to `null` if no constraints.
Note: `examples_file` is optional. If provided, read the file and use for few-shot learning.
Note: `profile_file` and `scoring_weights` are optional. If missing, use uniform weights.

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

{IF constraints exist, add this block:}
HARD CONSTRAINTS (ideas MUST satisfy ALL of these):
- Budget: ${budget_usd} max to build MVP
- Timeline: {timeline_weeks} weeks max to launch
- Required skills: {required_skills} (founder has these)
- Must include: {must_include}
- FORBIDDEN (never suggest): {forbidden}
- Solo developer: {solo_dev ? "Yes, must be buildable alone" : "Team OK"}

Ideas violating ANY constraint will be eliminated. Design within these limits.
{END IF}

{IF examples_file provided, read the file and add this block:}
DOMAIN EXPERTISE - Learn from these gold-standard examples:

{For each example in examples file, format as:}
EXAMPLE: {example.title}
- Audience: {example.facets.audience}
- Problem: {example.facets.jtbd}
- Differentiator: {example.facets.differentiator}
- Why it's good: {example.why_good}

Use these as style anchors. Match their specificity and depth. Your ideas should be at this quality level.
{END IF}

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

{IF constraints exist, add this block:}
CONSTRAINT CHECK (check FIRST, before scoring):
- Budget: ${budget_usd} max
- Timeline: {timeline_weeks} weeks max
- Required skills: {required_skills}
- Must include: {must_include}
- Forbidden: {forbidden}
- Solo dev only: {solo_dev}

If an idea violates ANY constraint, set "constraint_violation": true and "violation_reason": "..." in the output. These ideas will be auto-eliminated regardless of scores.
{END IF}

{IF examples_file provided, add this block:}
CALIBRATION BENCHMARKS - Use these reference scores:

{For each example in examples file, format as:}
BENCHMARK: {example.title}
- Scores: {example.scores} (overall: {example.overall_score})
- Why these scores: {example.score_rationale}

Calibrate your scoring against these benchmarks. A score of 7+ should indicate quality comparable to these examples.
{END IF}

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
    "constraint_violation": false
  },
  "idea-002": {
    ...
    "constraint_violation": true,
    "violation_reason": "Requires $50k+ infrastructure (exceeds $1000 budget)"
  }
}
```

For each idea:
1. If `constraint_violation: true`, set `overall_score = 0` and `status = "eliminated"`
2. Otherwise, compute `overall_score` as weighted sum:
   - `overall_score = Σ(score[k] * weight[k]) / Σ(weight[k])`
   - Note: criterion `risk` is scored as "high = less risky" (a benefit), so do NOT invert it

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

USER PRIORITIES (from profile weights): {top 2 weighted criteria}

WEAKEST AREAS (weighted): {top 2-3 criteria by weight[k] * (10 - score[k])}

BENCHMARK - Current best idea scores {best_score}/10

{IF constraints exist, add this block:}
CRITICAL CONSTRAINT GUARDRAILS:
Your refinement MUST still satisfy these original constraints:
- Timeline: {timeline_weeks} weeks max to build
- Solo developer: {solo_dev ? "Yes, must be buildable alone" : "Team OK"}
- Forbidden: {forbidden}

DO NOT add features that require:
- External hosting/infrastructure (CDN, web servers, databases beyond SQLite)
- Community/marketplace dynamics (moderation, discovery, ratings)
- Multi-month development effort
- Team coordination or external dependencies

Keep improvements FOCUSED. Improve weak scores without expanding scope.
{END IF}

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

### Phase 4.5: RE-SCORE REFINED IDEAS (required)

Reason: Tournament and preference-learning require rubric scores for the *active* ideas.

After REFINE, run CRITIC again on the current active set (which now includes refined ideas).

Use the same CRITIC prompt as Phase 2, but evaluate only the active ideas (including refined), then:

For each active idea:
1. If `constraint_violation: true`, set `overall_score = 0` and `status = "eliminated"`
2. Otherwise, set `scores = {…}` and compute `overall_score` using the same weighted sum formula
3. Store a short `judge_notes` string explaining the score highlights and weaknesses

Update `best_idea_id` and `best_score` after re-scoring.

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
  "constraints": { ... or null if none },
  "iterations_completed": N,
  "stop_reason": "...",
  "eliminated_by_constraints": N,
  "best_idea": { full idea object },
  "runner_up": { second best idea or null }
}
```

2. Present the winning idea to the user in this format:

---
{IF constraints exist:}
**Constraints applied:** budget ${budget_usd}, {timeline_weeks} weeks, skills: {required_skills}, must: {must_include}, no: {forbidden}, solo: {solo_dev}
{END IF}

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
- {IF constraints: "Satisfies all constraints"}

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
