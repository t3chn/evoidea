# Claude Code Instructions

This file contains project-specific instructions for Claude Code sessions.

## Tools & Workflow

- **Issue tracking:** `bd` (beads) - git-native, AI-friendly
- **Version control:** `jj` (jujutsu) - colocated with git
- **Pre-commit:** `uvx prek run --all-files`
- **Policy:** No Cyrillic in repo (until explicit i18n)

## Issue Format (bd create)

```
Goal: <one line>
Why: <1–3 bullets>
How: <1–3 bullets>
TDD: <what test we write first>
Done when: <1–3 bullets>
```

Example:
```bash
bd create --title="Add feature X" --type=feature --priority=2 --description="$(cat <<'EOF'
Goal: Enable users to do X

Why:
- Current workflow requires manual steps
- Users requested this in feedback

How:
- Add new CLI subcommand
- Parse args with clap
- Write output to runs/

TDD: Test that subcommand parses args and creates expected output file

Done when:
- Subcommand works with --help
- Output file created in correct location
- Tests pass
EOF
)"
```

## Commit Format (jj describe)

```
<what in one line>

Why:
- <1–3 bullets>

How:
- <1–3 bullets>

Refs: <bd-issue-id or link>
```

Example:
```bash
jj describe -m "$(cat <<'EOF'
Add preference profile export command

Why:
- Users lose tournament calibration between sessions
- No way to share preferences across team

How:
- New 'evoidea profile export' subcommand
- Serialize tournament weights to JSON
- Write to ~/.evoidea/profile.json

Refs: evoidea-78h
EOF
)"
```

## TDD Cadence

- Small slices: red → green → refactor
- Tests must be deterministic (mocks/fixtures, no network)
- One behavioral change per commit, tests in same change

## Session End (Landing the Plane)

**MANDATORY** - work is NOT complete until pushed:

```bash
# 1. Quality gates
uvx prek run --all-files
cargo test

# 2. Sync issues
bd sync

# 3. Push
jj git fetch
jj bookmark move main
jj git push --bookmark main

# 4. Verify
jj status
```

## jj Quick Reference

```bash
jj status                    # What's changed
jj log                       # History
jj new -m "summary"          # New change
jj describe -m "message"     # Update description
jj bookmark move main        # Move main to current
jj git push --bookmark main  # Push to remote
```

## bd Quick Reference

```bash
bd ready                     # Find available work
bd show <id>                 # View issue
bd create --title="..." --type=task --priority=2
bd update <id> --status=in_progress
bd close <id>
bd sync                      # Sync with git
```
