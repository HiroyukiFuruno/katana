---
name: release_workflow
description: katanaプロジェクトの公式リリース手順。make releaseによるバージョン更新からDMGビルド、GitHub Releaseまでのフローを定義。
---

# KatanA リリースワークフロー

本プロジェクト (Katana) におけるバージョンリリースの公式手順です。
`Cargo.toml` や `Info.plist` のバージョン表記ズレ（DMGとアプリ内バージョンの不一致など）を防ぐため、完全な自動化・一貫性チェックを含んだこのフローに必ず従ってください。

## ⚠️ 最重要ルール

- **権限の前提**: `make release` コマンドは `master` ブランチへの直接プッシュおよびタグのプッシュを伴うため、リポジトリの **Owner (Admin) 権限** が必要です。権限がない場合は利用できません。
- **手動でのタグ打ちは禁止**: `git tag vX.Y.Z` などを手動で打ってプッシュすることは固く禁止します。
- **手動でのCargo.tomlバージョン変更は非推奨**: リリース時は必ず `make release` を通じて更新してください。

## リリースフロー

### Step 1: リリース前準備 (Pre-flight checks)

リリース作業を始める前に、現在のローカル環境やファイルを準備します。

1. **GitHub Auth と GPG署名**: `gh auth status` や GPGキーの設定（GitHub へ公開されていること）が済んでいるかを確認します（`release.sh` 内で GPGキー検証も走ります）。
2. **CHANGELOG の更新**: リリース作業の前に、必ず `changelog-writing` スキル（`CHANGELOG.md` および `CHANGELOG.ja.md` の更新管理スキル）を実行し、今回のバージョンの変更履歴を追記してください。
3. **品質ゲートの事前確認**: 実行途中のエラーを防ぐため、`master` ブランチで `make check` が通ることを確認しておくことを推奨します。
4. **ブランチの統合確認 (⚠️ 必須)**: リリースは `master` が完全な状態でなければなりません。リリース前に、対象の変更に紐づく OpenSpec Base Feature Branch がすべて `master` へマージ済みであることを確認してください。

   ```bash
   # ローカルおよびリモートに未マージの feature branch が残っていないか確認
   git branch -a | grep -v master | grep -v HEAD
   # 想定: リリース対象の feature branch が出力されないこと
   ```

   未マージのブランチが残っている場合は、`openspec-branching` ワークフローの Step 5・Step 6 を先に実行してください。

### Step 2: リリースの実行 (Local / CI)

バージョン番号 (例: `0.1.7`) を指定して `make release` を実行します。
現在のスクリプト (`release.sh`) は、デフォルトで**Local リリース**を行います。CI（GitHub Actions）経由でビルドしたい場合は**CI リリース**のフラグを使用します。

#### パターンA: Local リリース (デフォルト・推奨)

ローカル環境で DMG のビルドから GitHub Release の発行、Homebrew Cask の更新までを一貫して行います。

```bash
make release VERSION=0.1.7
```

#### パターンB: CI リリース (GitHub Actions 依存)

タグをプッシュし、DMGビルドなど以降のリリース作業を GitHub Actions (CI) に委譲します。

```bash
make release VERSION=0.1.7 USE_GITHUB_WORKFLOW=1
```

> [!IMPORTANT]
> どちらのモードでも、コマンド実行時に内部で以下のステップが自動実行されます：
>
> 1. GPG署名キーの事前検証 (GitHub API)
> 2. `Cargo.toml`, `Cargo.lock`, `crates/*/Cargo.toml`, `crates/katana-ui/Info.plist` のバージョン自動更新
> 3. 品質ゲートの実行（`make check` によるカバレッジ・テスト強制）
> 4. `CHANGELOG.md` など更新対象のステージングとコミット (`chore: vX.Y.Z リリース準備`)
> 5. Git 注釈付き署名タグ (`vX.Y.Z`) の作成

### Step 3: リリース処理の完了後確認

それぞれのモードに応じた完了結果を確認します。

**Local リリースの場合:**
スクリプトがローカルで `make dmg` を実行・アップロードし、GitHub Release や Cask の更新を完了させます。最後に `master` と タグ の Origin 発行が行われます。ターミナルの標準出力および作成された Release の内容を確認してください。

**CI リリースの場合:**
タグと `master` ブランチの変更がすぐに Origin へ Push され、CI 上で GitHub Actions のリリースパイプラインが起動します。GitHub Actions の画面からパイプラインの成功と DMG 作成を確認してください。

**リリース後のブランチ衛生確認 (⚠️ 必須):**
リリース完了後、ローカルおよびリモートに orphan ブランチが残っていないことを確認してください。

```bash
git branch -a | grep -v master | grep -v HEAD
# 想定: feature branch が一切出力されないこと
```

残存している場合は `git branch -d <branch>` および `git push origin --delete <branch>` で即座に削除してください。

## 💡 トラブルシューティング（DMG・Info.plistのバージョンズレ防止策）

かつて `v0.1.3` 等の古いバージョン番号のまま DMG が作成されていた問題については、現在 `Makefile` の `package-mac` において、「cargo bundle でビルド生成された後に、`Cargo.toml` から抽出した 最新バージョン を `Info.plist` に強制注入する」仕組みを入れています。そのため、必ず `Cargo.toml` のバージョンと Finder 上のアプリ内バージョンは一致するようになり、再発しません。
