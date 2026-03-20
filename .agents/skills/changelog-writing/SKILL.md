---
name: changelog-writing
description: Decide whether a KatanA change belongs in the changelog and keep CHANGELOG.md and CHANGELOG.ja.md synchronized. Use when writing release notes, reviewing changelog scope, or translating changelog entries.
---

# KatanA CHANGELOG Writing

Use this skill for any task that touches `CHANGELOG.md` or `CHANGELOG.ja.md`, or decides whether a change deserves a release-note entry.
This file is the canonical repository-local version. If another AI agent needs the same skill under a different directory hierarchy, copy this content instead of maintaining a divergent variant.

## Workflow

1. Read [`docs/ai/changelog-policy.md`](../../../docs/ai/changelog-policy.md).
2. Default to app-facing changes only. If the shipped app, release artifact, or end-user workflow is unaffected, omit the entry.
3. Update `CHANGELOG.md` and `CHANGELOG.ja.md` together.
4. Keep version headings, section headings, and bullet structure aligned across both files.
5. Write concise, outcome-first bullets.

## Guardrails

- Do not add changelog entries for agent-skill updates, prompt/instruction changes, lint-only cleanup, CI-only maintenance, or tests with no app-side effect.
- Documentation and CI/CD changes are listed only when they materially affect users, release artifacts, or the release workflow.
- If the user explicitly asks to record an internal-only change, follow the request and still keep both changelog files synchronized.
