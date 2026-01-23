# evoidea — memetic idea evolver (local, Rust)

`evoidea` is a CLI tool that, given a user prompt:
1) generates a population of ideas,
2) scores them with a rubric (multi-criteria),
3) selects (elite + diversity),
4) crossovers and mutates,
5) refines the top ideas,
6) repeats for N iterations until a stop condition,
7) returns **exactly 1 best idea** and writes artifacts to `runs/<run_id>/`.

This bundle also includes a Codex skill: `.codex/skills/evo-ideator`.

## Quick start (mock, deterministic)
```bash
cargo run -- run --mode mock --prompt "Invent an AI-agent product that can reach $10k/mo quickly"
```

Output:
- `runs/<run_id>/config.json`
- `runs/<run_id>/state.json`
- `runs/<run_id>/history.ndjson`
- `runs/<run_id>/final.json`

## Real mode (via codex exec)
```bash
cargo run -- run --mode codex --search --prompt "Invent a product idea that can reach $10k/mo quickly"
```

## Documentation
See the `docs/` folder.
