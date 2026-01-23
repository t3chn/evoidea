#!/usr/bin/env bash
set -euo pipefail

PROMPT="${1:-}"
if [[ -z "${PROMPT}" ]]; then
  echo "Usage: run_evoidea.sh \"<prompt>\""
  exit 1
fi

# Prefer release if already built; otherwise fallback to cargo run.
if [[ -x "./target/release/evoidea" ]]; then
  ./target/release/evoidea run --mode codex --prompt "${PROMPT}"
else
  cargo run -- run --mode codex --prompt "${PROMPT}"
fi
