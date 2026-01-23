# non_md — files that must be materialized

This bundle is stored as `.md` files for portability.
The actual project needs real `.json/.sh/.csv/.mjs` files.

## What to do
1) Create these directories in the target repository:
- `schemas/`
- `.codex/skills/evo-ideator/scripts/`
- `evals/`

2) Materialize the files:

### Schemas
See `non_md/schemas/SCHEMAS.json.md` — it contains multiple blocks with `Target path:`.

### Skill script
Create:
- `.codex/skills/evo-ideator/scripts/run_evoidea.sh`
with contents from `non_md/codex_skill_scripts/run_evoidea.sh.md`

Do not forget:
- `chmod +x .codex/skills/evo-ideator/scripts/run_evoidea.sh`

### Evals (optional)
Create:
- `evals/evo-ideator.prompts.csv` from `non_md/evals/evo-ideator.prompts.csv.md`
- `evals/run-evals.mjs` from `non_md/evals/run-evals.mjs.md`
