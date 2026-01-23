# 08_EVALS — e2e checks via `codex exec --json` (optional)

## Idea
We do not judge \"idea quality\". We verify:
- the skill triggers
- artifacts are created
- output structure is respected
- the loop stops by the rules

## Dataset
`evals/evo-ideator.prompts.csv` (see `non_md/evals/*.md`)

## Runner
`evals/run-evals.mjs` (see `non_md/evals/*.md`)

## Note
`codex exec --json` prints newline-delimited JSON events.
