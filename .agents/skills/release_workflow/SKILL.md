---
name: release_workflow
description: katanaプロジェクトの公式リリース手順。make releaseによるバージョン更新からDMGビルド、GitHub Releaseまでのフローを定義。
---

# KatanA リリースワークフロー

本プロジェクト (Katana) におけるバージョンリリースの公式手順です。
`Cargo.toml` や `Info.plist` のバージョン表記ズレ（DMGとアプリ内バージョンの不一致など）を防ぐため、完全な自動化・一貫性チェックを含んだこのフローに必ず従ってください。

## ⚠️ 最重要ルール

- **手動でのタグ打ちは禁止**: `git tag vX.Y.Z` などを手動で打ってプッシュすることは固く禁止します。
- **手動でのCargo.tomlバージョン変更は非推奨**: リリース時は必ず `make release` を通じて更新してください。

## リリースフロー

### Step 1: リリース前検証 (Pre-flight checks)

リリース作業を始める前に、現在のブランチ (通常は `master`) で全チェックをパスするか確認します。

```bash
# 全フォーマット、lint、結合テスト、100%カバレッジの検証
make check
```

> [!NOTE]
> もし `make check` が通らない場合は、リリース作業を中断し、エラー（Lintエラー、テストカバレッジ不足など）を修正してください。

### Step 2: make release 実行
バージョン番号 (例: `0.1.7`) を指定して、`make release` を実行します。

```bash
make release VERSION=0.1.7
```

このコマンドは内部で以下を自動実行します：

1. `Cargo.toml` の workspace version を更新
2. `crates/katana-ui/Info.plist` の `CFBundleShortVersionString` を更新 (古いバージョンが残るバグを防止)
3. `CHANGELOG.md` を `vX.Y.Z` のタグ内容で自動更新 (git-cliff)
4. 更新後の内容に対して `make check` を実行し、品質ゲートを通過させる
5. 更新ファイル群をステージ・コミット (`chore: vX.Y.Z リリース準備`)
6. Gitの注釈付きタグ (`vX.Y.Z`) を作成

### Step 3: リモートへのプッシュ

コミットとタグをリモートにプッシュし、GitHub Actions によるリリースパイプラインを起動します。

```bash
git push origin master
git push origin vX.Y.Z
```

※ タグのプッシュによって `.github/workflows/release.yml` が自動トリガーされます。

### Step 4: パイプライン・DMGビルドの確認
CI上でDMG (`KatanA-Desktop-vX.Y.Z.dmg`) がビルドされ、自動的に GitHub Release が作成されます。
さらに、同時に `homebrew-katana` にある Cask (`Casks/katana-desktop.rb`) の自動更新も行われます。

1. GitHub リリースページに DMGファイルが添付されているか確認
2. Homebrew Cask の自動更新の確認

## 💡 トラブルシューティング（DMG・Info.plistのバージョンズレ防止策）

かつて `v0.1.3` 等の古いバージョン番号のまま DMG が作成されていた問題については、現在 `Makefile` の `package-mac` において、「cargo bundle でビルド生成された後に、`Cargo.toml` から抽出した 最新バージョン を `Info.plist` に強制注入する」仕組みを入れています。そのため、必ず `Cargo.toml` のバージョンと Finder 上のアプリ内バージョンは一致するようになり、再発しません。
