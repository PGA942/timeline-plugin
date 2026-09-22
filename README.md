# timeline-plugin — Unicode 順ソートノード(スロット `tl`)

簡略カスタマイズ UI / ノードグラフに **「Unicode 順」でソート** するアルゴリズムノードを 1 つ足すサンプル。
投稿を **Unicode コードポイント順**に並べ替える(数字 → 英字 → ひらがな → カタカナ → 漢字 → 絵文字の順)。
並び順(昇順 / 降順)と並べ替えのキー(本文 / 表示名 / ハンドル)は **ノードのパラメーター**で、
「ノードを設定」FOW に差し込まれる RRL 設定ページ(`tl.params`)からユーザーが選ぶ。
同じキーは `id` 昇順、それも同じなら入力順(stable)。降順はキーの比較だけ反転し、同点の `id` 昇順は変えない。

データ取得はホストの仕事で、このプラグインは **宣言・設定ページ・並べ替え**だけ。

## このリポジトリの形(`rasq` ワークスペース)

`rasq-sdk-sample` の `docs/cli.md` §2 のワークスペース。`rasq init` / `rasq new tl` が作る形に合わせてある。

```
rasq.toml                                # schema = "rasq.workspace/1"、[[plugins]] に name/slot/id
Cargo.toml                               # [workspace](rasq CLI が rasq.toml から作り直す)
kit/                                     # rasq-plugin-kit の写し(正典は SDK の rust/rasq-plugin-kit、D-244i)
plugins/timeline-plugin/
  manifest.json                          # plugin.manifest の答えそのもの
  src/lib.rs                             # manifest.json を include_str! で返し、tl.* を実装
dist/                                    # rasq build --raw の出力(.rasqpkg と生の .wasm)
```

- 契約の正典は [`CONTRACT.md`](CONTRACT.md) が指す `rasq-sdk-sample/docs/slots.md` §2.6。
- マニフェストは `plugins/timeline-plugin/manifest.json` の 1 か所だけ。`src/lib.rs` は `include_str!` で
  そのまま返すので、`rasq check` の往復検査(wasm の `plugin.manifest` == `manifest.json`)が常に一致する。
  名前 / 版 / 説明 / 権限を変えるときは `manifest.json` を直す。

## 契約(`docs/slots.md` §2.6)

| fn | args | 戻り |
|---|---|---|
| `plugin.manifest` | `{}` | `manifest.json`(`{id: "sample.timeline", kind: "view", slots: ["tl"], permissions: []}`) |
| `lifecycle` | `{kind, surface?}` | `{}` |
| `tl.timelines` | `{}` | `{timelines: [{id: "unicode-asc", label: "Unicode 順", base: "home"}]}`(`base` は互換用) |
| `tl.params` | `{id: "unicode-asc"}` | `{root: RRL Column, defaults: {order: "asc", key: "body"}}` |
| `tl.sort` | `{id, posts: [Post ワイヤ], params?}` | `{order: [postId…]}`(`id` 無しの投稿は除外) |
| `tl.params` / `tl.sort`(未知の `id`)/ `tl.customize` / その他 | | `C01` |

ホスト関数は使わない(`permissions: []`)。

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

## ビルドと確認

`rasq` CLI は `rasq-sdk-sample` から組む(`cd rust && cargo build --release -p rasq-cli`)。

```sh
rasq check                 # wasm32 ビルド → export 検査 → manifest 往復 → スキーマ / 登録簿 → cargo test
rasq test                  # cargo test だけ
rasq build --raw           # dist/sample.timeline-0.2.0.rasqpkg と dist/timeline_plugin.wasm
rasq inspect dist/sample.timeline-0.2.0.rasqpkg
```

`cargo test` は native で走るので、wasm でだけ起きる問題(時刻 / 乱数 / スレッド、`docs/cli.md` §5 の落とし穴)は
**ビルド済みの wasm をそのまま叩く** `rasq call` で確かめる:

```sh
rasq call timeline-plugin plugin.manifest
rasq call timeline-plugin tl.timelines
rasq call timeline-plugin tl.sort --args '{"id":"unicode-asc","params":{"order":"desc","key":"name"},
  "posts":[{"id":"p1","body":"う","author":{"displayName":"さくら","handle":"@zed"}},
           {"id":"p2","body":"あ","author":{"displayName":"たろう","handle":"@amy"}},
           {"id":"p3","body":"い","author":{"displayName":"Alice","handle":"@mia"}}]}'
# → {"order": ["p2", "p1", "p3"]}
```

## 端末での試し方(rasq-client)

1. `rasq dev` が出す絶対パス(または `dist/timeline_plugin.wasm` の絶対パス)を控える。
2. サイドバー → **WASMプラグイン試験 (dev)** → `.wasm のパスからインポート` に貼って **読み込み**。
3. **スロットに登録**(権限は無し)。
4. 簡略カスタマイズ UI / ノードグラフのアルゴリズム部品に **「Unicode 順でソート」** が増える。
5. **ノードを設定** を開くと「並び順」「並べ替えのキー」のラジオが出る。降順 + 表示名にすると、
   投稿が表示名の Unicode 降順(絵文字 → 漢字 → かな → 英数字)に並んでいれば OK。

## 掲載(審査)に出す

`rasq build` が出す `dist/sample.timeline-0.2.0.rasqpkg` がそのまま掲載できる形
(zstd tar + `bundle.json` `rasq.bundle/1` + `SHA512SUMS`)。`rasq publish --central <url> --dry-run` で
Central への 3 要求を書き出せる。開発者鍵の署名(`*.sig.json`)はサイドロード用で、リポジトリには入れない。
