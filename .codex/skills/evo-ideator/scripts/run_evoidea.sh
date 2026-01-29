#!/usr/bin/env bash
set -euo pipefail

PROMPT="${1:-}"
if [[ -z "${PROMPT}" ]]; then
  echo "Usage: run_evoidea.sh \"<prompt>\""
  exit 1
fi

if [[ -x "./target/release/evoidea" ]]; then
  EVOIDEA=(./target/release/evoidea)
else
  EVOIDEA=(cargo run --quiet --)
fi

# This repo's Rust CLI may not include the legacy `run` subcommand anymore.
if "${EVOIDEA[@]}" --help | grep -qE '^[[:space:]]+run[[:space:]]'; then
  "${EVOIDEA[@]}" run --mode codex --prompt "${PROMPT}"
  exit 0
fi

echo "error: this repo version does not support \`evoidea run\`." >&2
echo "hint: use the \`/evoidea\` command to generate run artifacts, then use \`evoidea show|export|tournament|profile\`." >&2
exit 2
