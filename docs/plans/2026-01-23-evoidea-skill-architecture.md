# Evoidea Skill-Based Architecture

**Date:** 2026-01-23
**Status:** Approved

## Overview

Evoidea evolves startup ideas using a memetic algorithm. Instead of calling LLM via subprocess, Claude Code executes the evolution loop directly using its tools and subagents.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   User                               │
│         /evoidea "Build AI startup ideas"           │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│              Claude Code + Skill                     │
│  ┌───────────────────────────────────────────────┐  │
│  │  /evoidea command (markdown instructions)     │  │
│  │  - reads prompt and parameters                │  │
│  │  - creates run_id, initializes state          │  │
│  │  - runs iteration loop                        │  │
│  └───────────────────────────────────────────────┘  │
│                      │                               │
│    ┌─────────────────┼─────────────────┐            │
│    ▼                 ▼                 ▼            │
│ [Generate]      [Critic]          [Refine]          │
│  subagent       subagent          subagents         │
│                                   (parallel)        │
└─────────────────────┬───────────────────────────────┘
                      │ Write tool
                      ▼
┌─────────────────────────────────────────────────────┐
│              runs/<run-id>/                          │
│  config.json, state.json, history.ndjson, final.json│
└─────────────────────────────────────────────────────┘
                      │ Read
                      ▼
┌─────────────────────────────────────────────────────┐
│              Rust CLI (utilities)                    │
│  evoidea show <id> | validate <id> | list           │
└─────────────────────────────────────────────────────┘
```

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Interface | Slash-command `/evoidea` | Seamless UX, native Claude Code integration |
| Parallelism | One subagent per task | Simpler debugging, less overhead |
| Rust code | Keep for utilities only | show/validate/list remain useful |
| State format | JSON in `runs/<run-id>/` | Compatible with existing schemas |
| Loop control | Skill manages full loop | Saves after each iteration, supports resume |

## Evolution Cycle (with Context Rot Mitigation)

```
Iteration N:
┌────────────────────────────────────────────────────┐
│ 1. GENERATE (if iteration == 1)                    │
│    - Subagent gets ONLY prompt                     │
│    - Clean context, no history                     │
│    - Generates population_size ideas               │
└────────────────────┬───────────────────────────────┘
                     ▼
┌────────────────────────────────────────────────────┐
│ 2. CRITIC                                          │
│    Context rot mitigation:                         │
│    - Pass ONLY active ideas (not pruned)           │
│    - Criteria at the beginning of prompt           │
│    - Each idea = compact JSON                      │
│    - No history of previous evaluations            │
└────────────────────┬───────────────────────────────┘
                     ▼
┌────────────────────────────────────────────────────┐
│ 3. SELECT (local logic, no LLM)                    │
│    - Pure math: sort + pick                        │
│    - No context needed                             │
└────────────────────┬───────────────────────────────┘
                     ▼
┌────────────────────────────────────────────────────┐
│ 4. REFINE (parallel, isolated)                     │
│    Context rot mitigation:                         │
│    - Each subagent sees ONE idea only              │
│    - + only its critique (not others)              │
│    - + brief summary of best idea (benchmark)      │
│    - Fresh context for each refine                 │
└────────────────────┬───────────────────────────────┘
                     ▼
┌────────────────────────────────────────────────────┐
│ 5. COMPRESS                                        │
│    Context rot mitigation:                         │
│    - Compress refined ideas to essential fields    │
│    - Remove verbose descriptions                   │
│    - History → only best_score trajectory          │
└────────────────────────────────────────────────────┘
```

### Context Rot Mitigation Principles

| Problem | Solution in evoidea |
|---------|---------------------|
| Long context degrades | Minimal context per call |
| Distractors hurt quality | Pruned ideas not passed to agents |
| Position matters | Criteria/instructions at prompt start |
| Multi-hop reasoning suffers | Separate tasks: critic ≠ refine |
| History accumulates noise | Compression after each iteration |

Sources:
- [Context Rot: Chroma Research](https://research.trychroma.com/context-rot)
- [Context rot: Understanding AI](https://www.understandingai.org/p/context-rot-the-emerging-challenge)

## Command Interface

```
/evoidea "Build tools for solo developers" --rounds 3 --population 6 --elite 2
```

### Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| prompt | (required) | Direction for idea generation |
| --rounds | 3 | Max iterations |
| --population | 6 | Ideas in first generation |
| --elite | 2 | Top-N for refine |
| --threshold | 9.0 | Score for early stop |
| --resume | - | Run ID to continue |
| --profile | - | Preference profile (exported by `evoidea profile export`) |

## Subagent Prompts

### Generate Subagent
```
Generate {population} startup/product ideas for: "{prompt}"

For each idea, return JSON:
- id: uuid
- title: 5-10 words
- summary: 2-3 sentences
- facets: {audience, jtbd, differentiator, monetization, distribution, risks}

Be diverse. No similar ideas.
```

### Critic Subagent
```
Rate each idea 0-10 on these criteria:
1. feasibility - can solo dev build MVP in weeks?
2. speed_to_value - time to first paying customer
3. differentiation - unique vs existing solutions
4. market_size - potential revenue scale
5. distribution - organic growth channels
6. moats - defensibility over time
7. risk - inverse of failure probability
8. clarity - how well-defined is the concept

Ideas to evaluate:
{compact_ideas_json}

Return: {idea_id: {scores}, ...}
```

### Refine Subagent
```
Improve this idea based on its weakest scores.

IDEA:
{idea_json}

CRITIQUE (weakest areas):
{weak_scores}

BENCHMARK (current best):
{best_idea_summary}

Return improved idea with same JSON structure + "changes" array.
```

## Data Format

### Directory Structure
```
runs/
└── <run-id>/
    ├── config.json      # Run parameters
    ├── state.json       # Current state (ideas, scores)
    ├── history.ndjson   # Event log (append-only)
    └── final.json       # Result (after completion)
```

### state.json
```json
{
  "run_id": "uuid",
  "iteration": 2,
  "ideas": [
    {
      "id": "uuid",
      "title": "DevTools Analytics",
      "summary": "...",
      "facets": { "audience": "...", ... },
      "scores": { "feasibility": 8, ... },
      "overall_score": 7.4,
      "status": "active",
      "origin": "refined",
      "parents": ["parent-uuid"]
    }
  ],
  "best_score": 7.4,
  "best_idea_id": "uuid",
  "stagnation_counter": 0
}
```

## Rust CLI Changes

| Before | After |
|--------|-------|
| `evoidea run` | Remove (now `/evoidea`) |
| `evoidea resume` | Remove (now `/evoidea --resume`) |
| `evoidea show <id>` | Keep |
| `evoidea validate <id>` | Keep |
| `evoidea list` | Add new |

## Stop Conditions

```
if best_score >= threshold:
    → STOP "threshold reached"

if stagnation_counter >= patience (default: 3):
    → STOP "stagnation"

if iteration >= max_rounds:
    → STOP "max rounds"
```

### Stagnation Detection
```
prev_best = state.best_score
... run iteration ...
new_best = max(idea.overall_score for active ideas)

if new_best > prev_best:
    stagnation_counter = 0
else:
    stagnation_counter += 1
```

## Error Handling

| Situation | Action |
|-----------|--------|
| Subagent returns invalid JSON | Retry once with format clarification |
| Subagent timeout | Log error, skip this idea in refine |
| Write fails | Stop immediately, save what exists |
| Resume with non-existent run-id | Clear error message |
| All ideas pruned (edge case) | Re-generate with new seed |

## Testing Strategy

| Level | What | How |
|-------|------|-----|
| Unit | JSON parsing, scoring math | Rust tests (existing) |
| Integration | Subagent prompts → valid output | Manual: run with `--rounds 1` |
| E2E | Full evolution cycle | Manual: `--rounds 2-3`, check final.json |
| Resume | Continue after interruption | Ctrl+C at iteration 2, then `--resume` |

## Migration Plan

1. Create `.claude/commands/evoidea.md`
2. Test skill manually (1 iteration)
3. Simplify Rust CLI (remove run/resume)
4. Test `evoidea show/validate`
5. Add `evoidea list`
6. Remove unused Rust code

## Files to Create

```
.claude/commands/evoidea.md    # Main command
```

## Files to Modify

```
src/main.rs         # CLI: show, validate, list only
src/orchestrator.rs # Keep only show_run, validate_run, add list_runs
```

## Files to Delete

```
src/llm.rs          # Fully remove
src/phase.rs        # Fully remove
```

## Files to Simplify

```
src/config.rs       # Read-only (no run params)
src/scoring.rs      # Only compute_overall_score (for validate)
src/storage.rs      # Read-only operations only
```
