#!/usr/bin/env bash
# Drift detector: maps recent file changes to affected specs
# Usage: tools/drift-detector.sh [num_commits]
# Default: checks last 5 commits

set -euo pipefail

COMMITS="${1:-5}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SPECS_DIR="$PROJECT_ROOT/docs/specs"

# Mapping: file path patterns -> spec files
declare -A PATTERN_TO_SPEC=(
    ["crates/myme-core/"]="core-auth.md"
    ["crates/myme-auth/"]="core-auth.md"
    ["crates/myme-services/"]="data-services.md"
    ["crates/myme-integrations/"]="data-services.md"
    ["crates/myme-organizations/"]="data-services.md"
    ["crates/myme-gmail/"]="google-services.md"
    ["crates/myme-calendar/"]="google-services.md"
    ["crates/myme-weather/"]="weather.md"
    ["crates/myme-ui/src/"]="ui-bridge.md"
    ["crates/myme-ui/qml/"]="qml-ui.md"
    ["crates/myme-ui/build.rs"]="ui-bridge.md"
)

# Get files changed in last N commits
changed_files=$(cd "$PROJECT_ROOT" && git diff --name-only "HEAD~${COMMITS}..HEAD" 2>/dev/null || echo "")

if [ -z "$changed_files" ]; then
    echo "No changes found in last $COMMITS commits."
    exit 0
fi

echo "Files changed in last $COMMITS commits:"
echo "$changed_files" | sed 's/^/  /'
echo ""

# Track which specs are affected
declare -A affected_specs

while IFS= read -r file; do
    for pattern in "${!PATTERN_TO_SPEC[@]}"; do
        if [[ "$file" == "$pattern"* ]]; then
            spec="${PATTERN_TO_SPEC[$pattern]}"
            affected_specs["$spec"]=1
        fi
    done
done <<< "$changed_files"

if [ ${#affected_specs[@]} -eq 0 ]; then
    echo "No specs affected by recent changes."
    exit 0
fi

echo "Specs that may need review:"
for spec in "${!affected_specs[@]}"; do
    spec_path="$SPECS_DIR/$spec"
    if [ -f "$spec_path" ]; then
        last_modified=$(stat -c %Y "$spec_path" 2>/dev/null || stat -f %m "$spec_path" 2>/dev/null)
        last_commit=$(cd "$PROJECT_ROOT" && git log -1 --format=%ct -- "docs/specs/$spec" 2>/dev/null || echo "0")
        days_since=$(( ($(date +%s) - last_commit) / 86400 ))
        echo "  $spec (last updated ${days_since}d ago)"
    else
        echo "  $spec (MISSING - needs creation)"
    fi
done
