# EvoIdea — Preference-Learning Loop (A)

Date: 2026-01-29
Status: Draft
Refs: evoidea-trn

## Goal
Use existing `tournament` + `profile` artifacts to *measurably* improve future EvoIdea runs by learning what the user prefers and feeding it back into:
- scoring (reweight criteria)
- refinement (optimize what the user actually values)
- selection (pick winners aligned with the user)

## Non-goals (MVP)
- Training a new LLM / RLHF / DPO on model weights.
- Ingesting private user files (repos, docs) as personalization input (that is track B).
- Perfectly modeling stylistic preferences not represented by the rubric (we start with rubric-aligned signal).

## Current State (repo reality)
- A run can store per-idea criterion scores (`feasibility`, `speed_to_value`, `differentiation`, `market_size`, `distribution`, `moats`, `risk`, `clarity`).
- `evoidea tournament` stores `runs/<run_id>/preferences.json`:
  - `comparisons[]`: `{ idea_a, idea_b, winner }`
  - `elo_ratings{ idea_id -> rating }`
- `evoidea profile export` wraps `preferences.json` into a portable JSON (version `1`), but today it does **not** compute generalizable signal for future runs.

Problem: Elo rankings are *idea-ID* specific to one run and do not generalize to new ideas. We need a representation that transfers: **criterion weights** and (optionally) a compact preference summary.

## Proposal: Derive Criterion Weights from Pairwise Comparisons

### Output contract (profile JSON)
Keep `version: 1` (backwards-compatible; `profile import` ignores new fields), but extend the exported profile with an optional `derived` block:

```json
{
  "version": 1,
  "created_at": "…",
  "source_run": "run-YYYYMMDD-HHMMSS",
  "stats": { "comparisons": 0, "ideas_rated": 0 },
  "preferences": { "comparisons": [], "elo_ratings": {} },
  "derived": {
    "criterion_weights": {
      "feasibility": 0.125,
      "speed_to_value": 0.125,
      "differentiation": 0.125,
      "market_size": 0.125,
      "distribution": 0.125,
      "moats": 0.125,
      "risk": 0.125,
      "clarity": 0.125
    },
    "fit": {
      "method": "pairwise-multiplicative-weights",
      "comparisons_used": 12,
      "holdout_accuracy": 0.67
    },
    "summary": [
      "Prioritizes fast time-to-value and distribution over moats.",
      "Penalizes high-risk ideas even when market size is large."
    ]
  }
}
```

### Learning algorithm (MVP)
We already have (a) pairwise preferences and (b) rubric scores for those ideas. Learn a weight vector `w` so that, for each comparison, the preferred idea tends to have a higher weighted score.

Use a deterministic, local-first online method that keeps weights positive:

1) Convert each idea’s rubric into a feature vector `f`:
   - Use the same criteria as the run scoring.
   - Ensure `risk` aligns with “higher is better”. If the run uses inverted risk in overall scoring, invert here too.

2) For each comparison `(winner, loser)` compute `Δ = f(winner) - f(loser)`.

3) Multiplicative update:
   - Initialize all weights to `1.0`.
   - For each criterion `i`: `w[i] = w[i] * exp(lr * Δ[i])`.
   - Clamp `w[i]` to a reasonable range (e.g. `[0.1, 10.0]`) to avoid blow-ups.
   - Normalize weights to sum to 1.

4) Report fit:
   - Deterministic train/test split (seeded), compute holdout accuracy:
     - predict winner by sign of `w · (f(a) - f(b))`.

Why this method:
- stable, tiny implementation surface (no external ML deps)
- monotone: if a criterion repeatedly correlates with winning, its weight increases
- positive weights avoid “rewarding worse scores”

### How weights are applied (next-run behavior)
Add an optional `--profile <file>` (or `--weights <file>`) to `/evoidea` (the agent command), and store the applied weights in `config.json`.

Then:
- **CRITIC phase**: still produces per-criterion scores.
- **Overall score**: compute `overall_score = weighted_sum(scores, weights)`.
- **SELECT phase**: rank by `overall_score` as before.
- **REFINE phase**: pick “weakest areas” by weighted deficiency, e.g. maximize `w[i] * (10 - score[i])`.
  - Include a short “User priorities” block in the refine prompt based on `derived.summary` (or just top-2 weights).

This is a closed loop:
`run -> tournament -> profile export -> next run uses derived weights -> higher preference-alignment`

## Metrics (measurable, local-first)
Minimum set:
- **Holdout accuracy** on past comparisons (stored in profile export).
- **Preference win-rate uplift**: on a new run, compare ranking by uniform weights vs learned weights on a small set of fresh pairwise comparisons (A/B test within one user session).
- **Stagnation reduction**: track stagnation counter vs baseline on runs using a profile (requires run logs).

## Risks / Limitations
- If rubric scores don’t correlate with user taste, weights won’t help much; rationales (below) become more important.
- Overfitting with very few comparisons; mitigate with a strong prior (start from uniform and cap weight changes).
- This models “what criteria matter” but not all stylistic preferences (e.g., “I only like B2B devtools”).

## Optional extension (later): capture rationales
During tournament, optionally ask: “Why did you pick A?” and capture:
- 1–2 free-text sentences, or
- a small tag set mapped to the 8 criteria

Use these rationales to:
- improve the critic/refiner prompts (“optimize for what the user said”)
- train weights even when the rubric is imperfect (semi-supervised)
