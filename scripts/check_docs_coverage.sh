#!/bin/bash
# Check that all CLI commands and flags are documented in README.md
#
# Extracts Commands:: variants and #[arg(long)] flags from main.rs
# and verifies each appears in README.md

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

missing_commands=()

for cmd in $commands; do
    # Convert PascalCase to lowercase for matching
    cmd_lower=$(echo "$cmd" | tr '[:upper:]' '[:lower:]')

    # Check if command appears in README (case-insensitive)
    if ! grep -qi "\b$cmd_lower\b" "$README"; then
        missing_commands+=("$cmd")
    fi
done

# Extract flags from #[arg(long)] patterns
# Looks for: #[arg(long)] followed by field_name: type
# Converts snake_case to --kebab-case
flags=$(awk '
    /#\[arg\(.*long.*\)]/ { getline; print }
' "$MAIN_RS" | \
    grep -oE '[a-z_]+:' | \
    sed 's/://' | \
    sed 's/_/-/g' | \
    sort -u)

missing_flags=()

for flag in $flags; do
    # Check if --flag appears in README
    if ! grep -q "\-\-$flag" "$README"; then
        missing_flags+=("--$flag")
    fi
done

has_errors=false

if [[ ${#missing_commands[@]} -gt 0 ]]; then
    echo "ERROR: CLI commands missing from README.md:"
    for cmd in "${missing_commands[@]}"; do
        echo "  - $cmd"
    done
    echo ""
    has_errors=true
fi

if [[ ${#missing_flags[@]} -gt 0 ]]; then
    echo "ERROR: CLI flags missing from README.md:"
    for flag in "${missing_flags[@]}"; do
        echo "  - $flag"
    done
    echo ""
    has_errors=true
fi

if [[ "$has_errors" == "true" ]]; then
    echo "Please document these in README.md before committing."
    exit 1
fi

echo "✓ All CLI commands and flags documented in README.md"
exit 0
