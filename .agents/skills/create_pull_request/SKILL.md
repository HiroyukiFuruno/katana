---
name: create_pull_request
description: PR creation workflow (GitHub CLI usage, template priority)
---

# PR Creation Workflow

Standard workflow for creating a PR. Prioritize repository-specific templates if they exist; otherwise, use the standard fallback template.

## Prerequisites

- GitHub CLI (`gh`) is installed and authenticated.
- Commits are completed on the working branch.

## Workflow

### 0. Determine Base Branch (MANDATORY)

> [!CAUTION]
> **NEVER hardcode `master`, `main`, or `develop` as the base branch.** Always derive it from context.

Determine the correct `--base` value:

1. If the current branch matches `<feature>-task<N>` pattern → base is `<feature>` (the Base Feature Branch per `/openspec-branching`).
2. If the current branch matches `<feature>-task<N>-<suffix>` pattern → base is still `<feature>`.
3. If the caller explicitly specifies a base branch, use that.
4. **If none of the above apply, ask the user.** Do not guess.

verify the base branch exists:

```bash
git branch -a | grep {base_branch}
```

### 0.5. Mandatory Self-Review (MANDATORY)

> [!IMPORTANT]
> **Before creating any PR, you MUST execute a self-review (via the `self-review` skill) and self-correct any detected issues.**
> This explicitly includes checking for violations against `docs/coding-rules.md` and `docs/coding-rules.ja.md`.
> Do NOT create a PR containing code that has not been proactively reviewed and fixed by you.

### 1. Verify Template Existence

Check if a template file exists within the repository.
Priority:

1. `.github/PULL_REQUEST_TEMPLATE.md`

### 2. Create the PR

#### A. When a template file exists

Read the contents of the template file and create the PR based on it.

```bash
# Check template content
cat .github/PULL_REQUEST_TEMPLATE.md

# Create PR — ALWAYS specify --base
gh pr create --base {base_branch} --head {branch_name} --title "{commit_message}" --body-file .github/PULL_REQUEST_TEMPLATE.md
```

*Tip: `--body-file` is convenient during `gh pr create`. Edit the contents appropriately.*

#### B. When no template file exists

Use the following standard template.

```markdown
<!-- I want to review in Japanese. -->

## 概要 (Overview)
<!-- Describe the overview or specify the JIRA ticket. -->

## 対応内容 (Changes Made)
<!-- Provide a bulleted list of implementation details. -->

## 影響範囲 (Impact Scope)
<!-- Provide a bulleted list of affected areas. -->

## 動作確認結果 (Verification Results)
<!-- Describe the verification results or attach screenshots. -->
```

Command execution example:

```bash
gh pr create --base {base_branch} --head {branch_name} --title "{commit_message}" --body "..."
```

## Important Notes

- **`--base` is mandatory.** Every `gh pr create` call must include it. Omitting `--base` causes GitHub CLI to default to the repository's default branch, which is almost always wrong for task branches.
- Ensure the title is clear and concise, referencing the commit message where appropriate.
- Include `<!-- I want to review in Japanese. -->` to explicitly request a Japanese language review.
