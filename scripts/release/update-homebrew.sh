#!/bin/zsh
# =============================================================================
# KatanA — Homebrew Cask Updater
# =============================================================================
# Usage: ./scripts/release/update-homebrew.sh <VERSION> <SHA256> <DMG_NAME>
# =============================================================================

set -euo pipefail

VERSION=$1
SHA256=$2
DMG_NAME=$3

if [[ -z "$VERSION" || -z "$SHA256" || -z "$DMG_NAME" ]]; then
    echo "Usage: $0 <VERSION> <SHA256> <DMG_NAME>" >&2
    exit 1
fi

VERSION_NUM="${VERSION#v}"

# Token Selection: Use HOMEBREW_KATANA_GIT_TOKEN (local) or HOMEBREW_TAP_TOKEN (CI)
TOKEN="${HOMEBREW_KATANA_GIT_TOKEN:-${HOMEBREW_TAP_TOKEN:-}}"

if [[ -z "$TOKEN" ]]; then
    echo "Error: Homebrew update token is not set." >&2
    echo "Please set HOMEBREW_KATANA_GIT_TOKEN (local) or ensure HOMEBREW_TAP_TOKEN is available (CI)." >&2
    exit 1
fi

CASK_PATH="Casks/katana-desktop.rb"
REPO="HiroyukiFuruno/homebrew-katana"

echo "[INFO] Fetching current Cask from $REPO..."
CURRENT=$(curl -s -H "Authorization: token $TOKEN" \
  "https://api.github.com/repos/$REPO/contents/$CASK_PATH")

# Validate API response
if ! printf "%s" "$CURRENT" | python3 -c "import sys,json; d=json.load(sys.stdin); sys.exit(0 if 'sha' in d else 1)" 2>/dev/null; then
    echo "Error: Failed to fetch Cask file from $REPO." >&2
    echo "$CURRENT" >&2
    exit 1
fi

FILE_SHA=$(printf "%s" "$CURRENT" | python3 -c "import sys,json; print(json.load(sys.stdin)['sha'])")

# Template Generation
CASK_CONTENT=$(cat <<EOF
cask "katana-desktop" do
  version "${VERSION_NUM}"
  sha256 "${SHA256}"

  url "https://github.com/HiroyukiFuruno/KatanA/releases/download/v#{version}/KatanA-Desktop-#{version}.dmg"
  name "KatanA Desktop"
  desc "Lightweight Markdown viewer with live preview, Mermaid diagrams, and syntax highlighting"
  homepage "https://github.com/HiroyukiFuruno/KatanA"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :ventura"

  app "KatanA Desktop.app"

  # Remove quarantine attribute (required for ad-hoc signed apps without Apple notarization)
  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-cr", "#{appdir}/KatanA Desktop.app"]
  end

  zap trash: [
    "~/Library/Preferences/com.katana.desktop.plist",
    "~/Library/Caches/com.katana.desktop",
  ]
end
EOF
)

# Base64 encode for GitHub API
ENCODED=$(echo "$CASK_CONTENT" | base64)

echo "[INFO] Updating Cask in $REPO..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X PUT \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  "https://api.github.com/repos/$REPO/contents/$CASK_PATH" \
  -d "{
    \"message\": \"chore: update katana-desktop to $VERSION_NUM\",
    \"content\": \"$ENCODED\",
    \"sha\": \"$FILE_SHA\"
  }")

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [[ "$HTTP_CODE" -ge 200 && "$HTTP_CODE" -lt 300 ]]; then
    URL=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('content',{}).get('html_url','unknown'))")
    echo "[OK] Updated Homebrew Cask: $URL"
else
    echo "Error: Failed to update Cask (HTTP $HTTP_CODE)." >&2
    echo "$BODY" >&2
    exit 1
fi
