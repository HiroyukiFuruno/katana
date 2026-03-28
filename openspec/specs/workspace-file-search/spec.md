## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: ファイル名検索モーダルの表示
ユーザが任意のショートカットキーまたはUI上のボタンを介して呼び出したとき、ワークスペース全体からファイルを検索するためのモーダル（ポップアップウィンドウ）を表示しなければならない。 The system SHALL conform.

#### Scenario: 検索モーダルを開く
- **WHEN** ユーザーがワークスペース検索用のアクションを実行する
- **THEN** 中央に検索入力を受け付けるモーダルウィンドウが表示される

### Requirement: 検索結果からのファイルオープン機能
モーダル内で単語を入力すると、現在読み込まれているワークスペース内のインデックス（キャッシュ等）から相対パスの部分一致（または正規表現）検索を行い、結果リストを表示しなければならない。 The system SHALL conform.
リストはパフォーマンス維持のため描画上限（例: トップ100件など）を設けなければならない。
リスト上のファイルアイテムをクリックすると、該当ファイルが存在するかを再確認した上で検索モーダルが閉じ、そのファイルが新たなタブで開かれなければならない（ファイルが削除されていた場合はモーダルを維持しエラー通知などの対応を行うこと）。

#### Scenario: 検索による上限付き結果表示
- **WHEN** 検索モーダルに "main" と入力し、200件の候補がマッチする
- **THEN** パフォーマンス維持のため、上位100件のみがリスト表示される

#### Scenario: 対象ファイルを開く
- **WHEN** 表示された検索結果のうち目的のファイルをクリックする
- **THEN** モーダルが閉じ、そのファイルが新しいタブで開かれ、カレントタブとしてフォーカスされる

#### Scenario: 削除されたファイルのオープン試行
- **WHEN** 検索候補クリック時にOS側で該当ファイルが削除されていた
- **THEN** ファイルを開かず（クラッシュせず）、ユーザに「ファイルが見つからない」旨の通知（UIへの表示等）を行う
