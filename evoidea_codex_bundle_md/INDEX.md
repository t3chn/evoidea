# Document bundle for implementing evoidea (Rust) + Codex skill

## Contents
- `AGENTS.md` — implementation instructions for a Codex CLI agent
- `docs/` — detailed spec, architecture, pipeline, CLI, and test plan
- `.codex/skills/evo-ideator/` — skill (markdown)
- `non_md/` — extra files stored as `.md` that must be materialized into real extensions

## How to use
1) Unpack this bundle into a new git repository.
2) Give a Codex CLI agent the instruction:
   - "Implement the project using AGENTS.md and docs/*.md; get to green cargo test."
3) Once the Rust code is ready, materialize files listed in `non_md/README.md`.
