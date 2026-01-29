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
0) Optional: discovery pre-flight (`--discover`)
   - If the user request includes `--discover` AND they did not provide explicit constraints (budget, timeline, skills, must, no, solo):
     - Ask 5 multiple-choice questions (collect all answers before proceeding):
       1) Skills/experience (choose 1+; comma-separated): 1) dev 2) design 3) marketing/growth 4) ai/ml 5) other (type your own)
       2) Time available to build MVP: 1) 4-8h 2) 10-16h 3) 20h+
       3) Business model: 1) saas 2) api 3) one-time 4) marketplace
       4) Target audience: 1) developers 2) business 3) creators 4) freelancers
       5) Tech approach: 1) llm-based 2) llm-assisted 3) no-llm
     - Write `runs/<run_id>/config.json` with:
       - `discovery`: normalized answers (strings + arrays)
       - `constraints` derived from discovery:
         - `timeline_weeks`: 4-8h → 1, 10-16h → 2, 20h+ → 4
         - `required_skills`: selected skill categories + "other" values as lowercase strings
         - `must_include`: selected business model token + audience token
         - `forbidden`: if tech approach is `no-llm`, include `["llm", "ai"]`
   - If explicit constraints are present, skip discovery (do not override user-provided constraints).
1) If the repo has the `evoidea` binary AND it supports the legacy `run` subcommand AND discovery is not requested:
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
