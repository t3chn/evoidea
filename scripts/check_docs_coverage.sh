#!/bin/bash
# Check that all CLI commands are documented in README.md
#
# Extracts Commands:: variants from main.rs and verifies each
# appears in README.md (case-insensitive)

set -e

MAIN_RS="src/main.rs"
README="README.md"

if [[ ! -f "$MAIN_RS" ]]; then
    echo "ERROR: $MAIN_RS not found"
    exit 1
fi

if [[ ! -f "$README" ]]; then
    echo "ERROR: $README not found"
    exit 1
fi

# Extract command names from enum Commands { ... } block
# Look for lines like "    List {" or "    Export {"
commands=$(awk '/^enum Commands/,/^}/' "$MAIN_RS" | \
           grep -E '^\s+[A-Z][a-z]+\s*\{' | \
           sed 's/[[:space:]]*\([A-Z][a-z]*\).*/\1/')

missing=()

for cmd in $commands; do
    # Convert PascalCase to lowercase for matching
    cmd_lower=$(echo "$cmd" | tr '[:upper:]' '[:lower:]')

    # Check if command appears in README (case-insensitive)
    if ! grep -qi "\b$cmd_lower\b" "$README"; then
        missing+=("$cmd")
    fi
done

if [[ ${#missing[@]} -gt 0 ]]; then
    echo "ERROR: CLI commands missing from README.md:"
    for cmd in "${missing[@]}"; do
        echo "  - $cmd"
    done
    echo ""
    echo "Please document these commands in README.md before committing."
    exit 1
fi

echo "✓ All CLI commands documented in README.md"
exit 0
