# CHANGELOG Policy

Use this policy whenever a task decides whether a change belongs in `CHANGELOG.md`, or when updating `CHANGELOG.md` / `CHANGELOG.ja.md`.

## Core Rule

- Default to app-facing release notes only.
- If a change does not affect the shipped desktop app, distributed artifact, upgrade path, or end-user workflow, do not add it to the changelog.
- Always update `CHANGELOG.md` and `CHANGELOG.ja.md` together.
- Keep both files aligned by version headings, section headings, and bullet structure.

## Include

- User-visible features, UX changes, and behavior changes
- User-facing bug fixes and regressions
- Settings, persistence, migration, or compatibility changes that users will notice
- Packaging, installer, signing, DMG, Homebrew, or release artifact changes that affect delivered builds
- Localization or platform support changes visible to users
- Documentation changes only when they materially affect end users or the release workflow

## Exclude By Default

- Internal refactors with no shipped behavior change
- CI-only, workflow-only, lint-only, or test-only maintenance
- AI agent skills, prompts, instructions, or workspace tooling updates
- Developer-only documentation cleanup
- Release-prep noise such as version bumps, staging-only edits, or housekeeping commits

## Exceptions

- If a docs or CI/CD change materially changes the release artifact, installation flow, update path, or user-facing guidance, it may be listed.
- If the user explicitly asks to record an internal-only change, follow the request.

## Writing Rules

- Write outcome-first bullets, not implementation diaries.
- Describe what changed for the user or for the shipped artifact.
- Keep bullets short and concrete.
- Avoid internal task chatter unless it helps identify a user-visible feature.
- Prefer one bullet per distinct shipped change.

## Section Choice

- `🚀 Features`: new user-visible capabilities
- `🐛 Bug Fixes`: fixes for user-facing defects or regressions
- `♻️ Refactoring`: only when the structural change is important enough to surface; otherwise omit it
- `📚 Documentation`: only for user-facing docs or release-process docs worth surfacing
- `👷 CI/CD`: only when build or release changes affect the delivered artifact or release reliability
- `🧪 Testing`, `🎨 Styling`, `🔧 Miscellaneous`, `⏪ Reverted`: use only when the shipped outcome is worth release-note space

## Sync Rules

- Add, remove, or reorder headings in both changelog files together.
- Keep English and Japanese entries semantically aligned. Translation does not need to be literal, but the meaning and structure must match.
- Ensure the release heading in `CHANGELOG.md` matches the workspace version in `Cargo.toml`.
- Do not update only one of the two changelog files.

## Repository-Specific Examples

Include:

- `Restore missing absolute path in metadata tooltip`
- `Add workspace search and filter support`
- `Inject app version into Info.plist during DMG builds`

Exclude:

- `Add changelog-writing skill`
- `Tighten markdownlint settings`
- `Refactor cache internals without behavior changes`
- `Rewrite tests with no app-side effect`

## Practical Checklist

Before adding a changelog bullet, confirm all of the following:

- The change affects the shipped app, release artifact, or end-user workflow.
- The bullet is written in the correct section.
- The same change is reflected in both `CHANGELOG.md` and `CHANGELOG.ja.md`.
- Heading structure stays aligned across both files.
