---
name: commit_and_push
description: Git commit & push workflow. Enforces pre-commit verification, separation of concerns, and Japanese commit messages.
---

# Commit & Push Skill

## ⚠️ Most Critical Rule

**Never execute a commit & push unless explicitly instructed or invoked via an active workflow like `openspec-delivery`.**

This rule overrides all others. The AI is prohibited from deciding to commit & push entirely on its own accord outside of defined boundaries.

## Basic Principles

### 1. Pre-commit Verification is Mandatory (General Rule)

Ensure all of the following verifications **succeed completely** before committing. Conditional passes are generally not allowed. (However, if **no implementation (code) changes** were made—e.g., modifying only documentation or OpenSpec files—verification can be skipped and `--no-verify` is permitted.)

**Verification Targets:**
- Static analysis (lint) of the current directory/project
- Unit Tests (UT)
- E2E Tests (if applicable)

Execution commands depend on the project structure. Examples:
```bash
make lint      # Static Analysis
make test      # UT
make e2e       # E2E
```

**Prohibited Actions:**
- "Implementation changed, some tests failed, but deemed irrelevant" → ❌ NOT permitted
- "Implementation changed, minor lint errors exist" → ❌ NOT permitted
- "Implementation changed, build errors exist in unrelated areas" → ❌ NOT permitted
- Using `git commit --no-verify` → Prohibited without explicit user permission if code was changed. Permitted only if changes are strictly documentation/specs.

### 2. Separate Commits by Concern

A single commit must contain exactly one concern.

**Good Examples:**
```text
fix: ヘルスチェックテストを復元
feat: probe サービスを必須化
refactor: SQS キュー設定を統一
```

**Bad Examples:**
```text
fix: 色々修正  ← Too vague
feat: probe追加とテスト修正とログ設定変更  ← Multiple concerns mixed
```

### 3. Commit Messages Must Be in Japanese

```text
feat: 新機能の追加
fix: バグ修正
refactor: リファクタリング
docs: ドキュメント更新
test: テスト追加・修正
chore: その他（ビルド設定等）
```

**Format:**
```text
<type>: <Concise Japanese description>

<Detailed explanation if necessary>
```

### 4. User Instructions Take Priority

Individual user instructions always override this skill document.
Examples:
- "Commit only, do not push" → Do not push
- "Combine it all into one commit" → Combine them
- "Write the message in English" → Write in English

## Workflow Procedure

### Step 1: Review Changes
```bash
git status --short
git diff --stat
```

### Step 2: Execute Verification
```bash
# Execute relevant verification commands
make lint
make test
# OR
npm run lint
npm run test
```
**If verification fails**: Do NOT commit. Fix errors and re-verify.

### Step 3: Stage by Concern
```bash
# Files for Concern A
git add path/to/fileA1 path/to/fileA2
git commit -m "fix: 関心事Aの修正"

# Files for Concern B
git add path/to/fileB1
git commit -m "feat: 関心事Bの追加"
```

### Step 4: Push (If no contrary instructions)
```bash
git push
```

## OpenSpec Integration

If the project root contains an `openspec/` directory and there is an active change under `openspec/changes/`, execute the following **mandatory checks before committing**.

### State Update (Every Time)
- Update checkboxes in `tasks.md` to reflect current progress
- Reflect any newfound or altered tasks into `tasks.md`

### Archive & Merge (When all tasks are complete)
If all tasks in `tasks.md` are completed, execute the OpenSpec "Archive Phase" **BEFORE** the final commit & push workflow:
1. Merge `openspec/changes/<feature-name>/spec.md` into `openspec/specs/`
2. Move `openspec/changes/<feature-name>/` to `openspec/archive/<feature-name>/`
3. Verify the `changes/` directory is cleaned up

Follow the procedures in the **openspec skill** (`openspec/SKILL.md` "3. Archive Phase").

## Checklist

Check before committing:
- [ ] All linting passes
- [ ] All tests pass
- [ ] Build succeeds (if applicable)
- [ ] Commits are separated by single concern
- [ ] Commit messages are appropriately written in Japanese
- [ ] User's specific instructions were followed
- [ ] **If active change exists in openspec/changes/**: state in tasks.md is fully up-to-date
- [ ] **If all tasks complete**: OpenSpec archive & merge is completely finished
