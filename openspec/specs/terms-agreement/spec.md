## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: 初回起動時の規約同意画面

アプリ初回起動時に利用規約の同意画面を表示し、同意を必須とする。同意するまでメインUIにアクセスできない。 The system SHALL conform.

#### Scenario: 初回起動で規約画面を表示

- **WHEN** ユーザーがアプリを初めて起動する（規約同意フラグが未設定）
- **THEN** スプラッシュスクリーンの後に利用規約の同意画面が全画面で表示される

#### Scenario: 規約への同意

- **WHEN** ユーザーが規約を確認し「同意する」ボタンをクリックする
- **THEN** 同意フラグが永続化され、メインUIに遷移する

#### Scenario: 同意しない場合

- **WHEN** ユーザーが「同意しない」ボタンをクリックする
- **THEN** アプリが終了する

#### Scenario: 2回目以降の起動

- **WHEN** ユーザーが規約に同意済みの状態でアプリを起動する
- **THEN** 規約画面はスキップされ、直接メインUIが表示される

### Requirement: 規約画面の言語選択プルダウン

規約同意画面に言語選択のプルダウンを配置し、規約内容と画面のUI言語を切り替えられるようにする。 The system SHALL conform.

#### Scenario: 言語選択プルダウンの表示

- **WHEN** 規約同意画面が表示される
- **THEN** 画面上部に言語選択プルダウン（English / 日本語）が表示される

#### Scenario: 言語切替による規約内容の切り替え

- **WHEN** ユーザーがプルダウンで言語を切り替える
- **THEN** 規約テキストとUIラベル（ボタン等）が選択した言語に即座に切り替わる

#### Scenario: 選択した言語の永続化

- **WHEN** ユーザーが言語を選択して規約に同意する
- **THEN** 選択した言語が設定に保存され、アプリ全体のデフォルト言語として使用される

### Requirement: 規約内容の管理

利用規約テキストはリソースファイルとして管理し、将来の更新を容易にする。 The system SHALL conform.

#### Scenario: 規約テキストのリソース管理

- **WHEN** 規約画面が表示される
- **THEN** 規約テキストはi18nリソースから読み込まれ、ハードコーディングされていない

#### Scenario: 規約バージョンの更新

- **WHEN** 規約のバージョンが更新される
- **THEN** 既存ユーザーに対して再同意が要求される（同意済みバージョンと最新バージョンの比較）
