#!/usr/bin/env bash
# Drift detector: maps recent file changes to affected specs
# Usage: tools/drift-detector.sh [num_commits]
# Default: checks last 5 commits
# Requires: Bash 4+ (for associative arrays)

set -euo pipefail

if ((BASH_VERSINFO[0] < 4)); then
    echo "Error: This script requires Bash 4.0 or later (found ${BASH_VERSION})." >&2
    echo "On macOS, install modern bash: brew install bash" >&2
    exit 1
fi

COMMITS="${1:-5}"

if ! [[ "$COMMITS" =~ ^[1-9][0-9]*$ ]]; then
    echo "Error: argument must be a positive integer, got '$COMMITS'" >&2
    echo "Usage: tools/drift-detector.sh [num_commits]" >&2
    exit 1
fi

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

# Check available commit count to handle shallow clones and small repos
commit_count=$(cd "$PROJECT_ROOT" && git rev-list --count HEAD 2>/dev/null || echo "0")

if [ "$commit_count" -eq 0 ]; then
    echo "Error: No commits found. Is this a valid git repository?" >&2
    exit 1
fi

if [ "$commit_count" -lt "$COMMITS" ]; then
    echo "Note: Only $commit_count commits available (requested $COMMITS). Checking all."
    COMMITS="$commit_count"
fi

# Get files changed in last N commits
changed_files=$(cd "$PROJECT_ROOT" && git diff --name-only "HEAD~${COMMITS}..HEAD" 2>&1)
if [ $? -ne 0 ]; then
    echo "Error: Failed to get changed files: $changed_files" >&2
    exit 1
fi

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
        last_commit=$(cd "$PROJECT_ROOT" && git log -1 --format=%ct -- "docs/specs/$spec" 2>/dev/null || echo "")
        if [ -z "$last_commit" ]; then
            echo "  $spec (never committed to git)"
        else
            days_since=$(( ($(date +%s) - last_commit) / 86400 ))
            echo "  $spec (last updated ${days_since}d ago)"
        fi
    else
        echo "  $spec (MISSING - needs creation)"
    fi
done
