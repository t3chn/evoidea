#!/usr/bin/env bash
set -euo pipefail

if ! command -v rg >/dev/null 2>&1; then
  echo "error: ripgrep (rg) is required for the no-cyrillic hook" >&2
  exit 2
fi

if [[ "$#" -eq 0 ]]; then
  exit 0
fi

found=0
for file in "$@"; do
  [[ -f "$file" ]] || continue

  if rg --pcre2 --no-heading --line-number --color=never '\p{Script=Cyrillic}' "$file" >/dev/null; then
    if [[ "$found" -eq 0 ]]; then
      echo "Cyrillic characters are not allowed in this repository." >&2
      echo "If you need i18n/localization files later, add an explicit exception then." >&2
      echo >&2
    fi

    echo "Found Cyrillic in: $file" >&2
    rg --pcre2 --no-heading --line-number --color=never '\p{Script=Cyrillic}' "$file" | head -n 20 >&2 || true
    echo >&2
    found=1
  fi
done

if [[ "$found" -ne 0 ]]; then
  exit 1
fi
