---
name: evo-ideator
description: Iteratively generate, research, evolve, and select the single best idea for a given prompt (memetic loop with scoring, crossover, mutation, and refinement).
metadata:
  short-description: Memetic idea evolver (best-of loop)
---

## When to use
Use this skill when the user asks to:
- invent a project / product / strategy
- solve a problem
- generate options and pick the best iteratively

## What it does
Runs an iterative memetic loop and returns:
- exactly ONE best idea (concise and practical) plus a score-based rationale

## Workflow
1) If the repo has the `evoidea` binary AND it supports the legacy `run` subcommand:
   - Run the materialized script in `scripts/run_evoidea.sh`.
2) Otherwise:
   - Do an instruction-only loop:
     - Generate 8–12 ideas
     - Score with rubric
     - Select top + diversity
     - Crossover/mutate
     - Refine top 2–3
     - Stop after 3–6 rounds or earlier if clear winner
   - Save artifacts under `runs/<run_id>/`

## User-facing response format (IDEATION)
- Title
- 5–10 bullets: problem, audience, solution, monetization, distribution, risks, why it won
No detailed implementation plan unless the user explicitly asks for it.

## References
- `references/RUNBOOK.md`
- `references/OUTPUT_FORMAT.md`
- `references/TROUBLESHOOTING.md`
