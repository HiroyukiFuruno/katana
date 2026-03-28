# Menu Enhancement

## Purpose
TBD (Enhanced application menus for Desktop environment)

## Requirements

### Requirement: About ダイアログの最適化
About メニューからアプリ情報を表示する。バージョン番号、ライセンス情報、著作権を含む。 The system SHALL conform.

#### Scenario: About ダイアログの表示
- **WHEN** ユーザーがメニューから「About」を選択する
- **THEN** アプリ名、バージョン番号（例: v0.1.0）、ライセンス（MIT）、著作権情報が表示される

#### Scenario: About ダイアログのアイコン表示
- **WHEN** About ダイアログが表示される
- **THEN** Katana Desktop のアイコンがダイアログ内に表示される

### Requirement: Help メニュー
Help メニューからGitHub Repositoryへ遷移する。 The system SHALL conform.

#### Scenario: GitHub Repository への遷移
- **WHEN** ユーザーがメニューから「Help」→「GitHubリポジトリ」を選択する
- **THEN** デフォルトブラウザでプロジェクトの GitHub Repository ページが開かれる

### Requirement: 寄付メニュー
寄付メニューを表示する。初版では「準備中」の表示のみ。 The system SHALL conform.

#### Scenario: 寄付メニューの表示
- **WHEN** ユーザーがメニューから「寄付」を選択する
- **THEN** 「寄付機能は準備中です」のメッセージが表示される
