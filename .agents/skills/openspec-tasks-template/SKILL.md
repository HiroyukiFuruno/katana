---
name: openspec-tasks-template
description: Add Katana-specific "Branch Rule" and "Release Process" boilerplate to OpenSpec `tasks.md` documents. Use this when instructed to create, extend, or organize `tasks.md`.
---
# Katana OpenSpec Tasks Template

When creating, formatting, or updating a `tasks.md` file for an OpenSpec change in the Katana project, ALWAYS use the following structures for the **Branch Rule** and the **Final Checklist**.
Throughout the development lifecycle, proactively utilize OpenSpec skills (`/opsx-propose`, `/opsx-explore`, `/opsx-apply`, `/opsx-archive`) to streamline the workflow.

## 1. Branch Rule (Initial Phase)

Insert this at the top of the `tasks.md` file, immediately after the DoR (Definition of Ready):

```markdown
## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.
```

*(Replace `[Change Directory Name]` with the actual change directory name, e.g., `desktop-viewer-polish-v0.5.0`)*

## 2. Final Verification & Release Work (Final Phase)

Insert this as the very last task group at the bottom of the `tasks.md` file.

```markdown
---

## x. Final Verification & Release Work

- [ ] x.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] x.2 Ensure `make check` passes with exit code 0
- [ ] x.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] x.4 Create a PR targeting `master`
- [ ] x.5 Merge into master (※ `--admin` is permitted)
- [ ] x.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `[Target Version]`
- [ ] x.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
```

*(Replace `x` with the next sequential task number. Replace `[Target Version]` with the actual target release version like `v1.2.3`)*

## 3. Mandatory Definition of Done (DoD) per Task

To prevent AI workflow skipping and ensure delivery consistency, you MUST INCLUDE the following unified checklist item in the Definition of Done for any individual task. Do NOT split it into separate manual steps. 

```markdown
### Definition of Done (DoD)
- [ ] (Other task-specific verifiable conditions...)
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
```
