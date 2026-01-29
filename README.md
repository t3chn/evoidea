```
███████╗██╗   ██╗ ██████╗ ██╗██████╗ ███████╗ █████╗
██╔════╝██║   ██║██╔═══██╗██║██╔══██╗██╔════╝██╔══██╗
█████╗  ██║   ██║██║   ██║██║██║  ██║█████╗  ███████║
██╔══╝  ╚██╗ ██╔╝██║   ██║██║██║  ██║██╔══╝  ██╔══██║
███████╗ ╚████╔╝ ╚██████╔╝██║██████╔╝███████╗██║  ██║
╚══════╝  ╚═══╝   ╚═════╝ ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝
```

<div align="center">

[![Typing SVG](https://readme-typing-svg.demolab.com?font=Fira+Code&weight=500&size=20&duration=2500&pause=800&color=00FF00&center=true&vCenter=true&width=520&lines=%24+evoidea+--evolve;memetic+algorithm+for+ideas;generate+%E2%86%92+critique+%E2%86%92+select+%E2%86%92+refine;ai-powered+startup+ideation)](https://git.io/typing-svg)

![License](https://img.shields.io/github/license/t3chn/evoidea?style=flat-square&label=license&labelColor=000000&color=00ff00)
![Rust](https://img.shields.io/badge/rust-stable-000000?style=flat-square&logo=rust&logoColor=00ff00)
![Claude](https://img.shields.io/badge/claude--code-skill-000000?style=flat-square&logo=anthropic&logoColor=00ff00)
![status](https://img.shields.io/badge/status-experimental-000000?style=flat-square&labelColor=000000&color=00ff00)

</div>

---

> This is a personal experiment exploring memetic algorithms for startup/product ideation. Not production-ready.

<details open>
<summary><b>📌 ~/problem</b></summary>
<br>

Brainstorming startup ideas is chaotic:

- ideas get lost or forgotten
- no systematic way to evaluate and compare
- hard to iterate on weak spots without losing what works
- constraints (budget, timeline, skills) often ignored

</details>

<details open>
<summary><b>🧠 ~/solution</b></summary>
<br>

Evoidea runs a **memetic algorithm** on ideas:

```
GENERATE → CRITIQUE → SELECT → REFINE → repeat
```

- **generate** diverse ideas from a prompt
- **critique** each idea on 8 dimensions (feasibility, differentiation, moats, etc.)
- **select** elite survivors, archive the weak
- **refine** winners by addressing their weakest scores
- **constraints** hard-filter ideas that violate budget/timeline/skills

Output: structured JSON with full evolution history, exportable to landing page format.

</details>

<details open>
<summary><b>🏗️ ~/architecture</b></summary>
<br>

```
evoidea/
├── .claude/skills/evoidea.md    # Claude Code skill (main interface)
├── src/                          # Rust CLI utilities
│   ├── main.rs                   # list, show, validate, export, tournament
│   └── lib.rs
├── runs/                         # evolution runs (gitignored)
│   └── run-YYYYMMDD-HHMMSS/
│       ├── config.json           # run parameters + constraints
│       ├── state.json            # current population + scores
│       ├── history.ndjson        # event log
│       └── final.json            # winner + runner-up
└── schemas/                      # JSON schemas for validation
```

**Two interfaces:**
- `/evoidea` skill in Claude Code (runs the evolution)
- `evoidea` CLI (inspect runs, export, tournament mode)

</details>

<details open>
<summary><b>🚀 ~/quickstart</b></summary>
<br>

### For Claude Code users

```bash
# clone and open in Claude Code
git clone https://github.com/t3chn/evoidea.git
cd evoidea
claude

# run evolution
/evoidea "Build developer productivity tools" --rounds 3 --population 6

# with constraints
/evoidea "SaaS for freelancers" --budget 1000 --timeline 4 --solo --no crypto,hardware

# with domain examples (few-shot learning)
/evoidea "Developer tools" --examples examples/devtools.json
```

### CLI utilities

```bash
# build CLI
cargo build --release

# list all runs
evoidea list
evoidea list --dir /path/to/runs  # custom runs directory

# show run results
evoidea show --run-id run-20260123-181141

# export to various formats
evoidea export --run-id run-20260123-181141 --preset landing
evoidea export --run-id run-20260123-181141 --preset decision-log
evoidea export --run-id run-20260123-181141 --preset stakeholder-brief
evoidea export --run-id run-20260123-181141 --preset changelog-entry

# visualize evolution tree
evoidea tree --run-id run-20260123-181141
evoidea tree --run-id run-20260123-181141 --format mermaid

# interactive tournament (rank ideas by preference)
evoidea tournament --run-id run-20260123-181141
evoidea tournament --run-id run-20260123-181141 --pairwise  # smart A/B mode (~2n comparisons)
evoidea tournament --run-id run-20260123-181141 --auto      # non-interactive, rank by score

# preference profiles (persist tournament calibration)
evoidea profile show --run-id run-20260123-181141
evoidea profile export --run-id run-20260123-181141 --output prefs.json
evoidea profile import --file prefs.json --run-id run-20260123-181141
```

</details>

<details open>
<summary><b>🤖 ~/agent-guide</b></summary>
<br>

### For AI agents using this skill

**Invocation:**
```
/evoidea "<prompt>" [--rounds N] [--population N] [--elite N] [--threshold N] [--profile FILE]
```

**Constraint flags:**
| Flag | Description | Example |
|------|-------------|---------|
| `--budget N` | Max USD for MVP | `--budget 500` |
| `--timeline N` | Max weeks to launch | `--timeline 4` |
| `--skills LIST` | Required skills (comma-sep) | `--skills rust,python` |
| `--must LIST` | Required elements | `--must api,cli` |
| `--no LIST` | Forbidden elements | `--no crypto,hardware` |
| `--solo` | Solo dev constraint (flag) | `--solo` |
| `--resume ID` | Continue from run | `--resume run-20260123-181141` |
| `--examples FILE` | Domain examples for few-shot | `--examples examples/devtools.json` |
| `--profile FILE` | Preference profile (from `evoidea profile export`) | `--profile prefs.json` |

**Evolution phases:**

1. **GENERATE** (iteration 1): Create `population_size` diverse ideas
2. **CRITIQUE**: Score each idea 0-10 on 8 criteria, check constraint violations
3. **SELECT**: Archive bottom half, keep top `elite_count`
4. **REFINE**: Improve elite ideas by addressing weakest scores

**Stop conditions:**
- `best_score >= threshold` (default 9.0)
- stagnation (no improvement for 3 rounds)
- `iteration >= max_rounds`

**Output structure:**
```
runs/<run_id>/
├── config.json    # parameters
├── state.json     # population + scores
├── history.ndjson # event log
└── final.json     # best_idea + runner_up + stop_reason
```

**Constraint enforcement:**
- Ideas violating ANY constraint get `overall_score = 0` and `status = "eliminated"`
- Constraints are checked BEFORE scoring in CRITIQUE phase
- Violation reason is logged for transparency

</details>

<details>
<summary><b>👤 ~/user-guide</b></summary>
<br>

### What you get

After running `/evoidea`, you'll have:

1. **Winning idea** with detailed facets:
   - audience, problem (JTBD), differentiator
   - monetization strategy, distribution channels
   - identified risks

2. **Score breakdown** on 8 dimensions:
   - feasibility, speed_to_value, differentiation
   - market_size, distribution, moats, risk, clarity

3. **Evolution history** showing how ideas improved

4. **Exportable formats** (`evoidea export --preset`):
   - `landing` → marketing landing page
   - `decision-log` → technical decision record
   - `stakeholder-brief` → executive summary
   - `changelog-entry` → release notes format

5. **Visualization** (`evoidea tree`):
   - ASCII tree showing parent→child evolution
   - Mermaid diagram for documentation

6. **Preference profiles** (`evoidea profile`):
   - Export/import tournament calibration
   - Share preferences across runs

### Tips for good prompts

```bash
# Too vague (bad)
/evoidea "Make money online"

# Specific direction (good)
/evoidea "Build tools for solo developers who ship side projects"

# With constraints (better)
/evoidea "Developer tools" --budget 500 --timeline 2 --solo --skills rust --no marketplace
```

### Understanding scores

| Score | Meaning |
|-------|---------|
| 8-10 | Exceptional, rare |
| 6-7 | Strong, worth pursuing |
| 4-5 | Average, needs work |
| 0-3 | Weak, likely archived |

The algorithm is intentionally harsh. A 7/10 is a good idea.

</details>

<details>
<summary><b>🧪 ~/dev</b></summary>
<br>

```bash
# run tests
cargo test

# build release
cargo build --release

# validate a run
evoidea validate --run-id <run_id>
```

</details>

---

<details>
<summary><b>🗒️ ~/notes</b></summary>
<br>

- Evolution runs are stored in `runs/` (gitignored by default)
- The skill uses Claude Code's Task tool to parallelize refinement
- Constraints are optional but recommended for realistic ideas
- Tournament mode helps calibrate your preferences for future runs (export a profile and pass it via `--profile`)
- Bundled examples: `examples/devtools.json`, `examples/saas.json`, `examples/consumer.json`

</details>

## License

MIT
