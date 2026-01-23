# 06_CLI — команды и флаги

## run
```bash
evoidea run \
  --prompt "<text>" \
  --mode mock|codex|command \
  [--search] \
  [--max-rounds 6] \
  [--population 12] \
  [--elite 4] \
  [--mutations 4] \
  [--crossovers 4] \
  [--wildcards 1] \
  [--threshold 8.7] \
  [--stagnation 2] \
  [--out runs/]
```

## resume
```bash
evoidea resume --run-id <uuid> [--max-rounds +2]
```

## show
```bash
evoidea show --run-id <uuid> [--format md|json]
```

## validate
```bash
evoidea validate --run-id <uuid>
```

## export
```bash
evoidea export --run-id <uuid> --format md
```

## Замечание про Codex CLI
`codex exec` обычно требует git-репозиторий; при необходимости используй `--skip-git-repo-check`.
