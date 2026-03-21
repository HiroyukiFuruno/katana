# Draw.io ファイルのマウント（推奨方法）

## 推奨：画像リンク構文で `.drawio` ファイルを直接マウント

Markdown の標準的な画像構文を使って、外部の `.drawio` ファイルを直接参照します。
コードブロック内に巨大なXMLを直書きするよりも可読性が高く、Obsidian や VS Code などのモダンな Markdown エディタでも広く採用されている方式です。

```markdown
![AWS 構成図](path/to/diagram.drawio)
```

### 実際の例

![AWS 構成図](aws.drawio)

---

## 参考：コードブロック構文（非推奨）

ファイルを別途用意せず Markdown 内に XML を直書きする場合は `` ```drawio `` フェンスを使います。
ただし、実際の Draw.io ファイルは数十〜数百行に及ぶためMarkdownの可読性が著しく低下します。
外部ファイルが使える場合は画像リンク構文を優先してください。

````markdown
```drawio
<mxfile ...>
  <diagram ...>
    <mxGraphModel ...>
      ...
    </mxGraphModel>
  </diagram>
</mxfile>
```
````
