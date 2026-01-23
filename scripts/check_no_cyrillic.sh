#!/usr/bin/env bash
set -euo pipefail

if ! command -v rg >/dev/null 2>&1; then
  echo "error: ripgrep (rg) is required for the no-cyrillic hook" >&2
  exit 2
fi

if [[ "$#" -eq 0 ]]; then
  exit 0
fi

is_translation_path() {
  local path="$1"

  case "$path" in
    locales/*|*/locales/*) return 0 ;;
    locale/*|*/locale/*) return 0 ;;
    i18n/*|*/i18n/*) return 0 ;;
    l10n/*|*/l10n/*) return 0 ;;
    translations/*|*/translations/*) return 0 ;;
    translation/*|*/translation/*) return 0 ;;
    *.po|*.pot|*.ftl|*.arb) return 0 ;;
  esac

  return 1
}

found=0
for file in "$@"; do
  [[ -f "$file" ]] || continue
  is_translation_path "$file" && continue

  if rg --pcre2 --no-heading --line-number --color=never '\p{Script=Cyrillic}' "$file" >/dev/null; then
    if [[ "$found" -eq 0 ]]; then
      echo "Cyrillic characters are not allowed outside translation files." >&2
      echo "Move localized strings to i18n files (e.g. locales/, i18n/) or remove Cyrillic." >&2
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
