#!/bin/zsh
# =============================================================================
# KatanA — macOS DMG Builder
# =============================================================================

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
APP_NAME="KatanA Desktop"
APP_BUNDLE="target/release/bundle/osx/${APP_NAME}.app"

# ── Colours ──────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────
info()    { echo "${CYAN}[INFO]${RESET}  $*"; }
success() { echo "${GREEN}[OK]${RESET}    $*"; }

# ── Argument Validation ───────────────────────────────────────────────────────
VERSION=$1
if [[ -z "$VERSION" ]]; then
    VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

DMG_NAME="KatanA-Desktop-${VERSION}.dmg"
DMG_OUT="target/release/${DMG_NAME}"

# ── Execution ─────────────────────────────────────────────────────────────────
info "Creating macOS DMG for v${VERSION}..."

rm -f "${DMG_OUT}"

if command -v create-dmg >/dev/null 2>&1; then
    info "Building DMG with create-dmg..."
    create-dmg \
        --no-gui \
        --volname "KatanA Desktop ${VERSION}" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "KatanA Desktop.app" 150 190 \
        --app-drop-link 450 190 \
        --no-internet-enable \
        "${DMG_OUT}" \
        "${APP_BUNDLE}"
else
    warn "create-dmg not found, falling back to hdiutil..."
    TMP_DMG=$(mktemp -d)/staging
    mkdir -p "$TMP_DMG"
    cp -R "${APP_BUNDLE}" "$TMP_DMG/"
    ln -s /Applications "$TMP_DMG/Applications"
    hdiutil create -volname "KatanA Desktop ${VERSION}" \
        -srcfolder "$TMP_DMG" -ov -format UDZO "${DMG_OUT}"
    rm -rf "$TMP_DMG"
fi

success "DMG created at ${DMG_OUT}"
