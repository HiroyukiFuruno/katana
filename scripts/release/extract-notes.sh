#!/bin/zsh
# =============================================================================
# KatanA — Release Notes Extractor
# =============================================================================
# Usage: ./scripts/release/extract-notes.sh <VERSION>
# =============================================================================

set -euo pipefail

VERSION=$1
if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <VERSION>" >&2
    exit 1
fi

# Strip 'v' prefix if present for matching in CHANGELOG.md
VERSION_NUM="${VERSION#v}"

# Extract notes using awk (logic from release.yml)
# Matches "## [x.y.z]" until the next "## ["
awk "/^## \[${VERSION_NUM}\]/{found=1; next} /^## \[/{if(found) exit} found" CHANGELOG.md
