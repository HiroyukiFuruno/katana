<p align="center">
  <img src="assets/icon.iconset/icon_128x128.png" width="128" alt="KatanA Desktop">
</p>

<h1 align="center">KatanA Desktop</h1>

<p align="center">
  A fast, lightweight Markdown workspace for macOS — built with Rust and egui.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml"><img src="https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/HiroyukiFuruno/katana/releases/latest"><img src="https://img.shields.io/github/v/release/HiroyukiFuruno/katana" alt="Latest Release"></a>
  <img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="Platform: macOS">
</p>

<p align="center">
  English | <a href="README.ja.md">日本語</a>
</p>

---

## What is KatanA

The name **KatanA** comes from the Japanese word **"刀" (katana)** — a razor-sharp blade forged with precision and purpose.

Just as a katana cuts cleanly through its target, this tool is designed to **slice through the complexity of documentation workflows** with speed and clarity. The name reflects the creator's desire to build something that helps developers **cut through challenges decisively**, one problem at a time.

KatanA Desktop is a fast, lightweight Markdown workspace for macOS, purpose-built for developers who work with specification documents and technical writing.

The trailing uppercase **A** stands for **"Agent"** — KatanA is designed for the era of AI agent-assisted development, where Markdown specifications serve as the bridge between humans and AI. **Katana × Agent = KatanA.**

---

## Background

As of **2026**, software development is rapidly evolving with the rise of **AI agents** assisting engineers in writing, reviewing, and maintaining code.

Alongside this shift, **Spec-Driven Development (SDD)** is gaining attention as a development methodology where specifications, architecture descriptions, and tasks are defined before implementation. These specifications are typically written in **Markdown documents** and serve as the central source of truth for both developers and AI agents.

However, existing Markdown tools are often either:

- general-purpose editors not optimized for technical documentation workflows, or
- heavy knowledge-management tools with unnecessary complexity.

**KatanA Desktop was created to solve this problem.**

The goal of KatanA is to provide a **simple, fast, and workspace-oriented Markdown environment** where developers can easily **browse and edit documentation used in SDD workflows**.

---

## Features

- **Live split-view preview** — Edit on the left, rendered HTML on the right, scroll-synced
- **Diagram rendering** — First-class support for Mermaid, PlantUML, and Draw.io fenced code blocks
- **GitHub Flavored Markdown** — Tables, strikethrough, task lists, footnotes, autolinks
- **Workspace-aware** — Open a folder and navigate files from the integrated file tree
- **Tab bar** — Multiple documents open simultaneously with VSCode-style tabs
- **i18n** — UI strings fully localised (English / Japanese bundled)
- **Fast native performance** — Built with Rust and egui, no Electron, no Node.js — just a single native binary

---

## Installation

> **macOS only** at this time. Apple Silicon and Intel are both supported.

### Download

1. Go to the [Releases page](https://github.com/HiroyukiFuruno/katana/releases/latest)
2. Download the latest `KatanA-Desktop-x.y.z.dmg`
3. Open the DMG and drag **KatanA Desktop.app** into your **Applications** folder

### First Launch (Important)

KatanA Desktop is ad-hoc signed but not notarized with Apple, so macOS will show an **"unidentified developer"** warning on first launch.

**Option A: Right-click to open (depends on macOS settings)**

1. Right-click (or Control-click) on **KatanA Desktop.app** in your Applications folder
2. Select **"Open"** from the context menu
3. Click **"Open"** in the confirmation dialog

**Option B: Command line (Recommended / Guaranteed to work)**

```sh
xattr -cr /Applications/KatanA\ Desktop.app
```

After the first successful launch, macOS will remember your choice and the app will open normally.

---

## First Release: v0.0.1

This release is the **first public version** of KatanA Desktop.

Version **v0.0.1** is intentionally minimal and focuses primarily on **Markdown viewing and workspace navigation**.

Key features currently include:

- Workspace-based Markdown browsing
- Diagram support (Mermaid / PlantUML / draw.io)
- Split preview with synchronized scrolling
- Fast native desktop performance (Rust-based)

This version should be considered an **MVP (Minimum Viable Product)** and the foundation for future development.

---

## Project Goals

KatanA aims to become a tool that helps developers:

- Read and navigate Markdown documentation efficiently
- Work with specification-driven workflows
- Integrate documentation with modern AI-assisted development

The long-term vision is to build a **lightweight documentation workspace** that complements modern development tools.

---

## We Want Your Ideas

This project is still in its early stages.

We welcome:

- feature ideas
- usability suggestions
- bug reports
- design feedback
- contributions from developers

If you have thoughts on how KatanA can improve the documentation workflow for developers, please open an [issue](https://github.com/HiroyukiFuruno/katana/issues) or [discussion](https://github.com/HiroyukiFuruno/katana/discussions).

Your feedback will directly influence the direction of the project.

---

## Open Source Commitment

KatanA Desktop is an open source project.

We are committed to keeping the **core functionality available for free**, especially features that do not incur ongoing operational costs.

These include:

- Markdown viewing
- Workspace navigation
- Documentation browsing
- Diagram rendering

---

## Future Plans

Some advanced features may require external services or operational costs.

For sustainability, the project may introduce:

- optional paid features (e.g., AI-assisted tools)
- small advertisements within the application

However, the **core documentation functionality will remain free**.

---

## For Developers

If you want to build from source, contribute, or understand the architecture:

- 📖 **[Development Guide](docs/development-guide.md)** — Setup, build, test, and project structure
- 📐 **[Coding Rules](docs/coding-rules.md)** — Code style, conventions, and quality gates
- 🏗️ **[Architecture Decisions](docs/adr/)** — Design rationale and ADRs

---

## Support the Project

If you find KatanA useful and would like to support its development, you can do so through sponsorship.

<a href="https://github.com/sponsors/HiroyukiFuruno"><img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors" alt="Sponsor"></a>

Support helps cover:

- development time
- infrastructure
- tooling costs

👉 **[Become a Sponsor](https://github.com/sponsors/HiroyukiFuruno)**

- ⭐ Star this repository — it helps others discover KatanA
- Share KatanA with people who might find it useful

---

## License

KatanA Desktop is released under the [MIT License](LICENSE).
