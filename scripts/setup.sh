#!/bin/zsh
# =============================================================================
# Katana — Development Environment Setup
# =============================================================================
# This script installs and configures all tools required to develop Katana:
#
#   - Homebrew        : macOS package manager
#   - git             : Latest Git (via Homebrew)
#   - rustup          : Rust toolchain manager
#   - Rust stable     : Compiler + clippy + rustfmt + llvm-tools-preview
#   - cargo-llvm-cov  : Code coverage (used in CI / pre-push gate)
#   - cargo-watch     : Auto-rebuild / auto-test on file changes
#   - cargo-outdated  : Detect stale Cargo dependencies
#   - cargo-bloat     : Analyse binary size (make bloat)
#   - tokei           : Count lines of source code (make loc)
#   - lefthook        : Git hooks runner (pre-commit / pre-push)
#
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

# ── Confirmation prompt ───────────────────────────────────────────────────────
confirm() {
  local prompt="${1:-Proceed?}"
  printf "%s [Y/n]: " "$prompt"
  read -r reply
  case "${reply:-Y}" in
    [Yy]*|"") return 0 ;;
    *)         return 1 ;;
  esac
}

# ── Banner ────────────────────────────────────────────────────────────────────
echo ""
echo "${BOLD}${CYAN}╔══════════════════════════════════════════════════════════════╗${RESET}"
echo "${BOLD}${CYAN}║         Katana — Development Environment Setup               ║${RESET}"
echo "${BOLD}${CYAN}╚══════════════════════════════════════════════════════════════╝${RESET}"
echo ""
echo "This script will check for and install the following:"
echo ""
echo "  ${BOLD}Package manager${RESET}"
echo "    • Homebrew"
echo ""
echo "  ${BOLD}Version control${RESET}"
echo "    • git (latest, via Homebrew)"
echo ""
echo "  ${BOLD}Rust toolchain${RESET}"
echo "    • rustup"
echo "    • Rust stable (clippy + rustfmt + llvm-tools-preview)"
echo ""
echo "  ${BOLD}Cargo extensions${RESET}"
echo "    • cargo-llvm-cov  (coverage — pre-push gate: 100% line)"
echo "    • cargo-watch     (make watch / make watch-run)"
echo "    • cargo-outdated  (make outdated)"
echo "    • cargo-bloat     (make bloat)"
echo ""
echo "  ${BOLD}Development utilities${RESET}"
echo "    • tokei           (make loc)"
echo "    • lefthook        (Git hooks: pre-commit + pre-push)"
echo ""

if ! confirm "Proceed with the installation?"; then
  echo "Aborted."
  exit 0
fi

# =============================================================================
# 1. Homebrew
# =============================================================================
header "Homebrew"

if command -v brew &>/dev/null; then
  success "Homebrew is already installed ($(brew --version | head -1))"
else
  info "Installing Homebrew..."
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

  # Add Homebrew to PATH for Apple Silicon / Intel
  if [[ -f "/opt/homebrew/bin/brew" ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
  elif [[ -f "/usr/local/bin/brew" ]]; then
    eval "$(/usr/local/bin/brew shellenv)"
  fi

  success "Homebrew installed successfully"
fi

# Ensure brew is up-to-date
info "Updating Homebrew..."
brew update --quiet
success "Homebrew updated"

# =============================================================================
# 2. git
# =============================================================================
header "git"

if brew list git &>/dev/null; then
  success "git (Homebrew) is already installed ($(git --version))"
else
  info "Installing git via Homebrew..."
  brew install git
  success "git installed ($(git --version))"
fi

# =============================================================================
# 3. rustup
# =============================================================================
header "rustup"

if command -v rustup &>/dev/null; then
  success "rustup is already installed ($(rustup --version 2>/dev/null | head -1))"
  info "Updating rustup..."
  rustup self update 2>/dev/null || warn "Could not self-update rustup (might be managed externally)"
else
  info "Installing rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  # Source the cargo env for the remainder of this script
  # shellcheck source=/dev/null
  source "${HOME}/.cargo/env"
  success "rustup installed"
fi

# Ensure cargo is on PATH for the rest of the script
if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  source "${HOME}/.cargo/env"
fi

# =============================================================================
# 4. Rust stable toolchain + components
# =============================================================================
header "Rust stable toolchain"

info "Installing / updating Rust stable..."
rustup toolchain install stable --no-self-update
rustup default stable

info "Adding required components: clippy, rustfmt, llvm-tools-preview..."
rustup component add clippy rustfmt llvm-tools-preview

success "Rust toolchain ready ($(rustc --version))"

# =============================================================================
# 5. cargo-llvm-cov
# =============================================================================
header "cargo-llvm-cov"

if cargo llvm-cov --version &>/dev/null; then
  success "cargo-llvm-cov is already installed ($(cargo llvm-cov --version))"
else
  info "Installing cargo-llvm-cov..."
  cargo install cargo-llvm-cov --locked
  success "cargo-llvm-cov installed"
fi

# =============================================================================
# 6. cargo-watch
# =============================================================================
header "cargo-watch"

if cargo watch --version &>/dev/null; then
  success "cargo-watch is already installed ($(cargo watch --version))"
else
  info "Installing cargo-watch..."
  cargo install cargo-watch --locked
  success "cargo-watch installed"
fi

# =============================================================================
# 7. cargo-outdated
# =============================================================================
header "cargo-outdated"

if cargo outdated --version &>/dev/null; then
  success "cargo-outdated is already installed ($(cargo outdated --version))"
else
  info "Installing cargo-outdated..."
  cargo install cargo-outdated --locked
  success "cargo-outdated installed"
fi

# =============================================================================
# 8. cargo-bloat
# =============================================================================
header "cargo-bloat"

if cargo bloat --version &>/dev/null; then
  success "cargo-bloat is already installed ($(cargo bloat --version))"
else
  info "Installing cargo-bloat..."
  cargo install cargo-bloat --locked
  success "cargo-bloat installed"
fi

# =============================================================================
# 9. tokei
# =============================================================================
header "tokei"

if command -v tokei &>/dev/null; then
  success "tokei is already installed ($(tokei --version))"
else
  info "Installing tokei via Homebrew..."
  brew install tokei
  success "tokei installed ($(tokei --version))"
fi

# =============================================================================
# 10. lefthook
# =============================================================================
header "lefthook"

if command -v lefthook &>/dev/null; then
  success "lefthook is already installed ($(lefthook version))"
else
  info "Installing lefthook via Homebrew..."
  brew install lefthook
  success "lefthook installed ($(lefthook version))"
fi

# Install hooks into the repo (idempotent)
if [[ -f "lefthook.yml" ]]; then
  info "Registering lefthook hooks in this repository..."
  lefthook install
  success "lefthook hooks installed (pre-commit + pre-push)"
else
  warn "lefthook.yml not found in current directory — skipping hook registration."
  warn "Run 'lefthook install' from the project root when ready."
fi

# =============================================================================
# Summary
# =============================================================================
echo ""
echo "${BOLD}${GREEN}╔══════════════════════════════════════════════════════════════╗${RESET}"
echo "${BOLD}${GREEN}║                 Setup complete! 🎉                           ║${RESET}"
echo "${BOLD}${GREEN}╚══════════════════════════════════════════════════════════════╝${RESET}"
echo ""
echo "Installed tools:"
echo "  brew         $(brew --version | head -1)"
echo "  git          $(git --version)"
echo "  rustc        $(rustc --version)"
echo "  cargo        $(cargo --version)"
echo "  clippy       $(cargo clippy --version)"
echo "  rustfmt      $(rustfmt --version)"
echo "  llvm-cov     $(cargo llvm-cov --version)"
echo "  cargo-watch  $(cargo watch --version)"
echo "  tokei        $(tokei --version)"
echo "  lefthook     $(lefthook version)"
echo ""
echo "Next steps:"
echo "  make build        # Build the workspace"
echo "  make test         # Run all tests"
echo "  make ci           # Full CI check (fmt + clippy + test)"
echo ""
echo "${YELLOW}Note:${RESET} If 'cargo' commands are not found, restart your shell or run:"
echo "  source \"\${HOME}/.cargo/env\""
echo ""
