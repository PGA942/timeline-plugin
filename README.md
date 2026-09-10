# timeline-plugin — Unicode 昇順 TL(`tl`)

ホーム TL に **「Unicode 昇順」** タブを足すカスタムタイムラインのサンプル。ホーム TL を餌(`base: "home"`)に、
投稿本文を **Unicode コードポイント昇順**で並べ替える(数字 → 英字 → ひらがな → カタカナ → 漢字 → 絵文字の順)。
同じ本文は `id` 昇順、それも同じなら入力順(stable)。

データ取得はホストの仕事で、このプラグインは **宣言と並べ替え**だけ(PROTOCOL.md §4.4 の v1)。

## 契約(CONTRACT.md §2.6 / PROTOCOL.md §4.4)

| fn | args | 戻り |
|---|---|---|
| `plugin.manifest` | `{}` | `{id: "sample.timeline", kind: "view", slots: ["tl"], permissions: []}` |
| `tl.timelines` | `{}` | `{timelines: [{id: "unicode-asc", label: "Unicode 昇順", base: "home"}]}` |
| `tl.sort` | `{id, posts: [Post ワイヤ]}` | `{order: [postId…]}`(本文の Unicode 昇順。`id` 無しの投稿は除外) |
| `tl.sort`(未知の `id`)/ `tl.customize` / その他 | | `C01` |

## ビルド / テスト

```sh
cargo test -p timeline-plugin
cargo build --release --target wasm32-unknown-unknown -p timeline-plugin
# → target/wasm32-unknown-unknown/release/timeline_plugin.wasm(./build.sh なら dist/)
```

## 端末での確認(rasq-client)

1. サイドバー → **WASMプラグイン試験 (dev)** → `.wasm のパスからインポート` に `timeline_plugin.wasm` の絶対パス → **読み込み**。
2. **スロットに登録**(権限は無し)。
3. ホーム TL のタブに **「Unicode 昇順」** が増える。開くと、ホーム TL と同じ投稿が
   本文の Unicode 順(英数字 → かな → 漢字 → 絵文字)に並んでいれば OK。

---

## このリポジトリの形

- クレートはルート(`src/`)、共通部品 [`rasq-plugin-kit`](kit) は `kit/` に同梱(開発用モノレポから写している)。
- 契約は [`CONTRACT.md`](CONTRACT.md)(スロットプラグイン契約 v0.1)。
- ビルド済み `.wasm` は [`dist/`](dist)。`cargo build --release --target wasm32-unknown-unknown` で更新。
