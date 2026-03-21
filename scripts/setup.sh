#!/bin/zsh
# =============================================================================
# KatanA — Development Environment Setup
# =============================================================================
# This script installs and configures all tools required to develop KatanA:
#
#   - Homebrew        : macOS package manager
#   - git             : Latest Git (via Homebrew)
#   - rustup          : Rust toolchain manager
#   - Rust stable     : Compiler + clippy + rustfmt + llvm-tools-preview
#   - cargo-llvm-cov  : Code coverage (used in CI / pre-push gate)
#   - cargo-watch     : Auto-rebuild / auto-test on file changes
#   - cargo-outdated  : Detect stale Cargo dependencies
#   - cargo-bloat     : Analyse binary size (make bloat)
#   - cargo-bundle    : macOS .app bundle packaging
#   - tokei           : Count lines of source code (make loc)
#   - lefthook        : Git hooks runner (pre-commit / pre-push)
#   - create-dmg      : macOS .dmg installer builder
#   - Optional AI skill copies from `.agents/skills` to other agent layouts
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
CANONICAL_SKILLS_DIR="${REPO_ROOT}/.agents/skills"

cd "${REPO_ROOT}"

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

copy_skills_to_layout() {
  local agent_name="$1"
  local dest_dir="$2"
  local skill_dir=""
  local skill_name=""
  local copied=0

  if [[ ! -d "${CANONICAL_SKILLS_DIR}" ]]; then
    warn "Canonical skills directory not found: ${CANONICAL_SKILLS_DIR}"
    return 0
  fi

  mkdir -p "${dest_dir}"

  for skill_dir in "${CANONICAL_SKILLS_DIR}"/*; do
    [[ -d "${skill_dir}" ]] || continue
    skill_name="$(basename "${skill_dir}")"
    rm -rf "${dest_dir}/${skill_name}"
    cp -R "${skill_dir}" "${dest_dir}/"
    copied=$((copied + 1))
  done

  if [[ ${copied} -eq 0 ]]; then
    warn "No skills found to copy from ${CANONICAL_SKILLS_DIR}"
    return 0
  fi

  success "Copied ${copied} skill(s) to ${agent_name} layout (${dest_dir})"
}

choose_menu_option() {
  local prompt="$1"
  shift

  local -a options=("$@")
  local selected=1
  local option_count=${#options[@]}
  local key=""
  local seq=""
  local first_render=1
  local i=0

  if [[ ! -t 0 || ! -t 1 ]]; then
    REPLY="1"
    info "Non-interactive terminal detected. Defaulting to option 1."
    return 0
  fi

  echo "${prompt}"
  echo "Use Up/Down arrows and press Enter."
  echo ""

  {
    while true; do
      if (( first_render == 0 )); then
        printf '\033[%dA' "${option_count}"
      fi

      for (( i = 1; i <= option_count; i++ )); do
        printf '\033[2K\r'
        if (( i == selected )); then
          printf "  ${BOLD}${CYAN}> %s${RESET}\n" "${options[i]}"
        else
          printf "    %s\n" "${options[i]}"
        fi
      done

      first_render=0
      read -rsk1 key

      case "${key}" in
        ""|$'\n'|$'\r')
          REPLY="${selected}"
          echo ""
          return 0
          ;;
        $'\x1b')
          read -rsk2 seq
          case "${seq}" in
            "[A") (( selected = selected == 1 ? option_count : selected - 1 )) ;;
            "[B") (( selected = selected == option_count ? 1 : selected + 1 )) ;;
          esac
          ;;
        [1-9])
          if (( key >= 1 && key <= option_count )); then
            selected="${key}"
          fi
          ;;
      esac
    done
  } always {
    printf '\033[?25h'
  }
}

prompt_skill_copy_layout() {
  local choice=""

  header "Optional AI Skill Layout Copy"
  echo "Canonical repository-local skills live in:"
  echo "  ${CANONICAL_SKILLS_DIR}"
  echo ""

  choose_menu_option \
    "Select the optional target layout to prepare:" \
    "1) Antigravity only (skip copy) [recommended]" \
    "2) Codex (.codex/skills)" \
    "3) Claude Code (.claude/skills)" \
    "4) GitHub Copilot / VS Code (.github/skills)" \
    "5) All supported layouts"
  choice="${REPLY:-1}"

  case "${choice}" in
    1)
      info "Skipping skill copy. Antigravity remains the primary local setup."
      ;;
    2)
      copy_skills_to_layout "Codex" "${REPO_ROOT}/.codex/skills"
      ;;
    3)
      copy_skills_to_layout "Claude Code" "${REPO_ROOT}/.claude/skills"
      ;;
    4)
      copy_skills_to_layout "GitHub Copilot / VS Code" "${REPO_ROOT}/.github/skills"
      ;;
    5)
      copy_skills_to_layout "Codex" "${REPO_ROOT}/.codex/skills"
      copy_skills_to_layout "Claude Code" "${REPO_ROOT}/.claude/skills"
      copy_skills_to_layout "GitHub Copilot / VS Code" "${REPO_ROOT}/.github/skills"
      ;;
  esac
}

# ── Banner ────────────────────────────────────────────────────────────────────
echo ""
echo "${BOLD}${CYAN}╔══════════════════════════════════════════════════════════════╗${RESET}"
echo "${BOLD}${CYAN}║         KatanA — Development Environment Setup               ║${RESET}"
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
echo "    • cargo-bundle    (make package-mac)"
echo ""
echo "  ${BOLD}Development utilities${RESET}"
echo "    • tokei           (make loc)"
echo "    • lefthook        (Git hooks: pre-commit + pre-push)"
echo "    • create-dmg      (make dmg)"
echo ""
echo "  ${BOLD}Optional AI agent setup${RESET}"
echo "    • Copy repository-local skills from .agents/skills to other agent layouts"
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
# 8b. cargo-bundle
# =============================================================================
header "cargo-bundle"

if cargo bundle --version &>/dev/null; then
  success "cargo-bundle is already installed ($(cargo bundle --version))"
else
  info "Installing cargo-bundle..."
  cargo install cargo-bundle
  success "cargo-bundle installed"
fi

# =============================================================================
# 8c. git-cliff
# =============================================================================
header "git-cliff"

if command -v git-cliff &>/dev/null; then
  success "git-cliff is already installed ($(git-cliff --version))"
else
  info "Installing git-cliff..."
  cargo install git-cliff
  success "git-cliff installed"
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

# =============================================================================
# 10b. create-dmg
# =============================================================================
header "create-dmg"

if command -v create-dmg &>/dev/null; then
  success "create-dmg is already installed ($(create-dmg --version 2>&1 | head -1))"
else
  info "Installing create-dmg via Homebrew..."
  brew install create-dmg
  success "create-dmg installed"
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

prompt_skill_copy_layout

# =============================================================================
# 11. nvm & Node.js
# =============================================================================
header "nvm & Node.js"

export NVM_DIR="$HOME/.nvm"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  success "nvm is already installed"
  source "$NVM_DIR/nvm.sh"
else
  info "Installing nvm..."
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
  source "$NVM_DIR/nvm.sh"
  success "nvm installed"
fi

if command -v node &>/dev/null && node -v | grep -q 'v24'; then
  success "Node.js v24 is already installed ($(node -v))"
else
  info "Installing Node.js v24 (LTS)..."
  nvm install 24
  nvm use 24
  nvm alias default 24
  success "Node.js installed ($(node -v))"
fi

# =============================================================================
# 12. OpenSpec
# =============================================================================
header "OpenSpec"

if command -v openspec &>/dev/null; then
  success "OpenSpec CLI is already installed ($(openspec --version))"
else
  info "Installing OpenSpec CLI globally..."
  npm install -g @fission-ai/openspec
  success "OpenSpec CLI installed"
fi

if [[ -f ".openspec.yaml" || -f "openspec.yaml" || -d "openspec" ]]; then
  success "OpenSpec is already initialized in this repository"
else
  info "Initializing OpenSpec..."
  npx @fission-ai/openspec init --yes || true
  success "OpenSpec initialized"
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
