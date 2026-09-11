# timeline-plugin — Unicode 順ソートノード(`tl`、契約 v0.2)

簡略カスタマイズ UI / ノードグラフに **「Unicode 順」でソート** するアルゴリズムノードを 1 つ足すサンプル。
投稿を **Unicode コードポイント順**に並べ替える(数字 → 英字 → ひらがな → カタカナ → 漢字 → 絵文字の順)。
並び順(昇順 / 降順)と並べ替えのキー(本文 / 表示名 / ハンドル)は **ノードのパラメーター**で、
「ノードを設定」FOW に差し込まれる RRL 設定ページ(`tl.params`)からユーザーが選ぶ。
同じキーは `id` 昇順、それも同じなら入力順(stable)。降順はキーの比較だけ反転し、同点の `id` 昇順は変えない。

データ取得はホストの仕事で、このプラグインは **宣言・設定ページ・並べ替え**だけ。

## 契約(CONTRACT.md §2.6)

| fn | args | 戻り |
|---|---|---|
| `plugin.manifest` | `{}` | `{id: "sample.timeline", kind: "view", slots: ["tl"], permissions: []}` |
| `tl.timelines` | `{}` | `{timelines: [{id: "unicode-asc", label: "Unicode 順", base: "home"}]}`(`base` は互換用) |
| `tl.params` | `{id: "unicode-asc"}` | `{root: RRL Column, defaults: {order: "asc", key: "body"}}` |
| `tl.sort` | `{id, posts: [Post ワイヤ], params?}` | `{order: [postId…]}`(`id` 無しの投稿は除外) |
| `tl.params` / `tl.sort`(未知の `id`)/ `tl.customize` / その他 | | `C01` |

### パラメーター(`tl.params` の設定ページ)

| セクション | パラメーター(ノード key) | 選択肢 | 既定 |
|---|---|---|---|
| 並び順 | `order` | `asc`(昇順)/ `desc`(降順) | `asc` |
| 並べ替えのキー | `key` | `body`(本文)/ `name`(表示名 = `author.displayName`)/ `handle`(ハンドル = `author.handle`) | `body` |

設定ページは `SectionHeader` と `RadioListTile` だけで組む。各ラジオは
`{"t":"RadioListTile","k":"<パラメーター名>","p":{"title":"…","value":"<選択肢>","group":"$.settings.<パラメーター名>"},"on":{"change":"settings.set"}}`
で、値は Bind `$.settings.<key>`、書き込みは `settings.set`(ホストがノードのパラメーターに保存する)。

`tl.sort` の `params` は保存されたパラメーター(無ければホストが `defaults` を渡す)。`params` 自体が無い /
未知の値なら `asc` / `body` として扱う。キーが取れない投稿(`author` 無しなど)は空文字として並ぶ。

## ビルド / テスト

```sh
cargo test -p timeline-plugin
cargo build --release --target wasm32-unknown-unknown -p timeline-plugin
# → target/wasm32-unknown-unknown/release/timeline_plugin.wasm(./build.sh なら dist/)
```

## 端末での確認(rasq-client)

1. サイドバー → **WASMプラグイン試験 (dev)** → `.wasm のパスからインポート` に `timeline_plugin.wasm` の絶対パス → **読み込み**。
2. **スロットに登録**(権限は無し)。
3. 簡略カスタマイズ UI / ノードグラフのアルゴリズム部品に **「Unicode 順でソート」** が増える。
4. **ノードを設定** を開くと「並び順」「並べ替えのキー」のラジオが出る。降順 + 表示名にすると、
   投稿が表示名の Unicode 降順(絵文字 → 漢字 → かな → 英数字)に並んでいれば OK。

---

## このリポジトリの形

- クレートはルート(`src/`)、共通部品 [`rasq-plugin-kit`](kit) は `kit/` に同梱(開発用モノレポから写している)。
- 契約は [`CONTRACT.md`](CONTRACT.md)(スロットプラグイン契約 v0.1)。
- ビルド済み `.wasm` は [`dist/`](dist)。`cargo build --release --target wasm32-unknown-unknown` で更新。
