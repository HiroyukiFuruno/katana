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

# Create PR (Fill template content into body and open in editor, or replace content and specify via CLI)
gh pr create --base develop --head {branch_name} --title "{commit_message}" --body-file .github/PULL_REQUEST_TEMPLATE.md
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
gh pr create --base develop --head {branch_name} --title "{commit_message}" --body "<!-- I want to review in Japanese. -->

## 概要
<!-- Describe the overview or specify the JIRA ticket. -->

## 対応内容
<!-- Provide a bulleted list of implementation details. -->

## 影響範囲
<!-- Provide a bulleted list of affected areas. -->

## 動作確認結果
<!-- Describe the verification results or attach screenshots. -->"
```

## Important Notes

- Ensure the title is clear and concise, referencing the commit message where appropriate.
- Include `<!-- I want to review in Japanese. -->` to explicitly request a Japanese language review.
