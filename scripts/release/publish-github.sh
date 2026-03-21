#!/bin/zsh
# =============================================================================
# KatanA — GitHub Publisher
# =============================================================================
# Usage: ./scripts/release/publish-github.sh <VERSION> <DMG_PATH> <NOTES_PATH>
# =============================================================================

set -euo pipefail

VERSION=$1
DMG_PATH=$2
NOTES_PATH=$3

if [[ -z "$VERSION" || -z "$DMG_PATH" || -z "$NOTES_PATH" ]]; then
    echo "Usage: $0 <VERSION> <DMG_PATH> <NOTES_PATH>" >&2
    exit 1
fi

if [[ ! -f "$DMG_PATH" ]]; then
    echo "Error: DMG file not found at $DMG_PATH" >&2
    exit 1
fi

if [[ ! -f "$NOTES_PATH" ]]; then
    echo "Error: Notes file not found at $NOTES_PATH" >&2
    exit 1
fi

# Ensure checksums.txt is created
cd "$(dirname "$DMG_PATH")"
DMG_NAME=$(basename "$DMG_PATH")
shasum -a 256 "$DMG_NAME" > checksums.txt

echo "[INFO] Creating GitHub Release $VERSION..."
gh release create "$VERSION" \
    --title "KatanA Desktop $VERSION" \
    --notes-file "$NOTES_PATH" \
    "$DMG_NAME" \
    "checksums.txt"

echo "[OK] GitHub Release $VERSION created and artifacts uploaded."
