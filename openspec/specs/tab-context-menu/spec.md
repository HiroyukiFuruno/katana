# Tab Context Menu

## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: タブコンテキストメニュー - 閉じる操作
タブを右クリックした際にコンテキストメニューを表示し、各種閉じる操作を提供する。 The system SHALL conform.

#### Scenario: 「閉じる」でアクティブタブを閉じる
- **WHEN** ユーザーがタブを右クリックし「閉じる」を選択する
- **THEN** そのタブが閉じられる

#### Scenario: 「その他を閉じる」で他タブを閉じる
- **WHEN** ユーザーがタブを右クリックし「その他を閉じる」を選択する
- **THEN** 右クリックしたタブ以外の全タブが閉じられる

#### Scenario: 「全てを閉じる」で全タブを閉じる
- **WHEN** ユーザーがタブを右クリックし「全てを閉じる」を選択する
- **THEN** 全てのタブが閉じられる

#### Scenario: 「右を閉じる」で右側のタブを閉じる
- **WHEN** ユーザーがタブを右クリックし「右を閉じる」を選択する
- **THEN** 右クリックしたタブより右にある全タブが閉じられる

#### Scenario: 「左を閉じる」で左側のタブを閉じる
- **WHEN** ユーザーがタブを右クリックし「左を閉じる」を選択する
- **THEN** 右クリックしたタブより左にある全タブが閉じられる

### Requirement: タブのドラッグ並び替え
タブをドラッグ&ドロップして並び順を変更できる。 The system SHALL conform.

#### Scenario: タブのドラッグ移動
- **WHEN** ユーザーがタブをドラッグし、別の位置にドロップする
- **THEN** タブがドロップ位置に移動し、他のタブが適切にシフトする

### Requirement: ピン留めタブ
タブをピン留めして固定位置に配置できる。ピン留めタブはアイコンのみのコンパクト表示となり、閉じる系操作の対象外となる。 The system SHALL conform.

#### Scenario: タブのピン留め
- **WHEN** ユーザーがタブを右クリックし「ピン留め」を選択する
- **THEN** タブがタブバーの左端に移動し、コンパクト表示になる

#### Scenario: ピン留め解除
- **WHEN** ユーザーがピン留めされたタブを右クリックし「ピン留め解除」を選択する
- **THEN** タブが通常のサイズに戻り、通常の位置に移動する

### Requirement: 最近閉じたタブの復元
最近閉じたタブを復元する機能を提供する。 The system SHALL conform.

#### Scenario: 閉じたタブの復元
- **WHEN** ユーザーがコンテキストメニューから「閉じたタブを復元」を選択する
- **THEN** 最後に閉じたタブが復元される（閉じた逆順で復元、最大10件保持）
