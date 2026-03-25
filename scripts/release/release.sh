#!/bin/zsh
# =============================================================================
# KatanA — Release Automation
# =============================================================================
# This script automates the versioning, tagging, and pushing of a new release.
# Usage: ./scripts/release/release.sh <VERSION>
# =============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────
info()    { echo "${CYAN}[INFO]${RESET}  $*"; }
success() { echo "${GREEN}[OK]${RESET}    $*"; }
warn()    { echo "${YELLOW}[WARN]${RESET}  $*"; }
error()   { echo "${RED}[ERROR]${RESET} $*" >&2; }
header()  { echo "\n${BOLD}${CYAN}==> $*${RESET}"; }

confirm() {
  if [[ "${FORCE:-0}" = "1" ]]; then
    return 0
  fi
  local prompt="${1:-Proceed?}"
  printf "%s [Y/n]: " "$prompt"
  read -r reply
  case "${reply:-Y}" in
    [Yy]*|"") return 0 ;;
    *)         return 1 ;;
  esac
}

# ── Argument Validation ───────────────────────────────────────────────────────
VERSION=${1:-$VERSION}
USE_GITHUB_WORKFLOW=${USE_GITHUB_WORKFLOW:-${2:-0}}
FORCE=${FORCE:-${3:-0}}

if [[ -z "$VERSION" ]]; then
    error "VERSION is required. Usage: scripts/release/release.sh x.y.z [USE_GITHUB_WORKFLOW=1] [FORCE=1]"
    exit 1
fi

header "Releasing v${VERSION}"
if [[ "$USE_GITHUB_WORKFLOW" = "1" ]]; then
    info "Mode: GitHub Actions (CI) Release requested (USE_GITHUB_WORKFLOW=1)"
else
    info "Mode: Local Release"
fi

# ── 0. Check for existing tag ─────────────────────────────────────────────────
TAG_EXISTS=false
if git rev-parse "v${VERSION}" >/dev/null 2>&1; then
    warn "Tag v${VERSION} already exists."
    if confirm "Use existing tag as-is?"; then
        TAG_EXISTS=true
    else
        error "Aborting release to avoid tag conflict."
        exit 1
    fi
fi

# ── 1. GPG Preflight ──────────────────────────────────────────────────────────
check_gpg() {
    info "Checking GPG signing key..."
    
    local SIGNING_FORMAT
    SIGNING_FORMAT=$(git config --get gpg.format || true)
    if [[ -n "$SIGNING_FORMAT" && "$SIGNING_FORMAT" != "openpgp" ]]; then
        info "Skipping GitHub GPG key preflight (gpg.format=$SIGNING_FORMAT)"
        return 0
    fi
    
    local SIGNING_KEY
    SIGNING_KEY=$(git config --get user.signingkey || true)
    if [[ -z "$SIGNING_KEY" ]]; then
        info "Skipping GitHub GPG key preflight (user.signingkey not set)"
        return 0
    fi
    
    local REMOTE_URL GITHUB_OWNER
    REMOTE_URL=$(git remote get-url origin)
    GITHUB_OWNER=$(echo "$REMOTE_URL" | sed -E 's#(git@github.com:|https://github.com/)##; s#\.git$##; s#/.*##')
    if [[ -z "$GITHUB_OWNER" ]]; then
        info "Skipping GitHub GPG key preflight (origin is not GitHub)"
        return 0
    fi
    
    if ! command -v gh >/dev/null 2>&1; then
        info "Skipping GitHub GPG key preflight (gh not installed)"
        return 0
    fi
    
    local PUBLIC_GPG_KEYS
    if ! PUBLIC_GPG_KEYS=$(gh api "users/$GITHUB_OWNER/gpg_keys" --jq '.[].key_id' 2>/dev/null); then
        info "Skipping GitHub GPG key preflight (could not query GitHub API)"
        return 0
    fi
    
    SIGNING_KEY=$(echo "$SIGNING_KEY" | tr '[:lower:]' '[:upper:]')
    if ! echo "$PUBLIC_GPG_KEYS" | grep -Fx "$SIGNING_KEY" >/dev/null 2>&1; then
        error "GPG key $SIGNING_KEY is not published on GitHub user $GITHUB_OWNER."
        error "Add the public key in GitHub Settings > SSH and GPG keys before running release."
        exit 1
    fi
    success "GPG signing key verified."
}
check_gpg

# ── 2. Quality Gates ──────────────────────────────────────────────────────────
info "Running quality gates (make check)..."
if ! make check; then
    error "Quality gate failed. Please fix errors before releasing."
    exit 1
fi

# ── 3. Update version files ───────────────────────────────────────────────────
info "Updating version in Cargo.toml..."
sed -i '' 's/^version = ".*"/version = "'"${VERSION}"'"/' Cargo.toml

info "Syncing Cargo.lock..."
cargo check --workspace >/dev/null 2>&1 || true

info "Updating version in Info.plist..."
perl -i -0pe 's/(<key>CFBundleShortVersionString<\/key>\s*<string>).*?(<\/string>)/$1v'"${VERSION}"'$2/' crates/katana-ui/Info.plist

# ── 3.5. Validate CHANGELOG via AST Linter ────────────────────────────────────
info "Validating CHANGELOG.md using AST Linter..."
if ! cargo test -p katana-linter --test ast_linter ast_linter_changelog_contains_current_workspace_version -q >/dev/null 2>&1; then
    error "AST Linter failed: Version v${VERSION} not found in CHANGELOG.md."
    error "Please update CHANGELOG.md (and CHANGELOG.ja.md) with the new release notes and run 'make release' again."
    # Discard version bumps to keep repository clean for the retry
    git checkout Cargo.toml Cargo.lock crates/*/Cargo.toml crates/katana-ui/Info.plist 2>/dev/null || true
    exit 1
fi

if ! grep -q "^## \[${VERSION}\]" CHANGELOG.ja.md; then
    warn "Version v${VERSION} not found in CHANGELOG.ja.md. You may want to update the Japanese changelog."
fi

# ── 4. Commit and Tag ─────────────────────────────────────────────────────────
info "Staging release changes..."
git add Cargo.toml Cargo.lock crates/*/Cargo.toml crates/katana-ui/Info.plist CHANGELOG.md CHANGELOG.ja.md

if git diff --cached --quiet; then
    warn "Nothing new to commit (version files might be already up-to-date)."
else
    info "Committing release changes..."
    git commit -m "chore: v${VERSION} リリース準備"
    success "Release commit created."
fi

if [[ "$TAG_EXISTS" = "false" ]]; then
    info "Creating signed tag v${VERSION}..."
    git tag -s "v${VERSION}" -m "Release v${VERSION}"
    success "Tag v${VERSION} created."
else
    info "Reusing existing tag v${VERSION}."
fi

# ── 5. Push and/or Publish ────────────────────────────────────────────────────
if [[ "$USE_GITHUB_WORKFLOW" = "1" ]]; then
    # CI Release Path
    if confirm "Push to origin to trigger CI release?"; then
        info "Pushing changes and tag..."
        git push origin HEAD --no-verify
        git push origin "v${VERSION}" --no-verify
        
        info "Triggering GitHub Actions release workflow..."
        gh workflow run release.yml -f tag="v${VERSION}"
        success "GitHub Actions release workflow triggered."
    else
        warn "Push skipped. You must push manually to trigger the release."
    fi
else
    # Local Release Path
    header "Building and Publishing locally..."
    
    info "Building DMG locally..."
    make dmg

    DMG_PATH=$(find target/release -name "KatanA-Desktop-*.dmg" -maxdepth 1 | sort -V | tail -1)
    if [[ -z "$DMG_PATH" ]]; then
        error "Local DMG build failed or file not found."
        exit 1
    fi
    success "Local DMG built: $DMG_PATH"

    ZIP_PATH="target/release/KatanA-macOS.zip"
    rm -f "$ZIP_PATH"
    info "Building ZIP locally ($ZIP_PATH)..."
    APP_PATH=$(find target/release/bundle/osx -name "*.app" -maxdepth 1 | head -1)
    if [[ -n "$APP_PATH" ]]; then
        (cd "$(dirname "$APP_PATH")" && zip -r -q "../../KatanA-macOS.zip" "$(basename "$APP_PATH")")
        success "Local ZIP built: $ZIP_PATH"
    else
        error "Local APP bundle not found for zipping."
        exit 1
    fi

    RELEASE_NOTES_PATH="/tmp/RELEASE_NOTES_${VERSION}.md"
    info "Extracting release notes..."
    ./scripts/release/extract-notes.sh "$VERSION" > "$RELEASE_NOTES_PATH"
    
    if confirm "Publish to GitHub and Update Homebrew now?"; then
        info "Pushing changes and tag to origin..."
        git push origin HEAD --no-verify
        git push origin "v${VERSION}" --no-verify

        ./scripts/release/publish-github.sh "$VERSION" "$DMG_PATH" "$RELEASE_NOTES_PATH"
        
        SHA256=$(shasum -a 256 "$DMG_PATH" | awk '{print $1}')
        DMG_NAME=$(basename "$DMG_PATH")
        ./scripts/release/update-homebrew.sh "$VERSION" "$SHA256" "$DMG_NAME"
        
        success "Local release and publication complete!"
    else
        warn "Publication skipped. You must publish manually."
    fi
    
    rm -f "$RELEASE_NOTES_PATH"
fi

success "Release v${VERSION} process finished! 🚀"
