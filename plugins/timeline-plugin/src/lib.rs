//! timeline-plugin — カスタム TL(`tl`)= アルゴリズムノードのサンプル(契約 v0.13、docs/slots.md §2.6)。
//!
//! 「Unicode 順」ソートノードを 1 つ提供する。並び順(昇順/降順)と並べ替えのキー(本文/表示名/ハンドル)は
//! `tl.params` が返す RRL 設定ページでユーザーが決め、ホストがノードのパラメーターとして保存し、
//! `tl.sort` の `params` で渡してくる。データ取得はホストの仕事で、このプラグインは
//! 宣言(`tl.timelines`)・設定ページ(`tl.params`)・並べ替え(`tl.sort`)だけ。
//!
//! `plugin.manifest` の答えは同じディレクトリの `manifest.json` そのもの(`include_str!`。
//! `rasq check` の往復検査が常に一致する)。
use rasq_plugin_kit::{err, rasq_plugin, rrl, serde_json::{json, Value}, unsupported};

const TL_UNICODE: &str = "unicode-asc";

/// パラメーター名(= RRL ノードの key、`$.settings.<key>` の key)。
const PARAM_ORDER: &str = "order";
const PARAM_KEY: &str = "key";
const DEFAULT_ORDER: &str = "asc";
const DEFAULT_KEY: &str = "body";

/// `plugin.manifest` の答え = `manifest.json`(名前 / 版 / 説明 / 権限はあちらを直す)。
fn manifest() -> Value {
    rasq_plugin_kit::serde_json::from_str(include_str!("../manifest.json")).expect("manifest.json")
}

fn root() -> Value {
    rrl::column("root", vec![rrl::text("desc", "カスタム TL サンプル: 本文/表示名/ハンドルを Unicode 順に並べるソートノード「Unicode 順」")])
}

fn handle(fn_name: &str, args: Value) -> Result<Value, Value> {
    match fn_name {
        "plugin.manifest" => Ok(manifest()),
        "lifecycle" => Ok(json!({})),
        "tl.timelines" => Ok(json!({
            "timelines": [{"id": TL_UNICODE, "label": "Unicode 順", "base": "home"}]
        })),
        "tl.params" => params(&args),
        "tl.sort" => sort(&args),
        _ => Err(unsupported(fn_name)),
    }
}

rasq_plugin! { scope: "sample.timeline:main", root: root, call: handle }

fn check_id(args: &Value) -> Result<(), Value> {
    let id = args.get("id").and_then(Value::as_str).unwrap_or("");
    if id == TL_UNICODE {
        Ok(())
    } else {
        Err(err("C01", format!("unknown timeline: {id:?}")))
    }
}

// ---------------------------------------------------------------------------
// tl.params — 設定 DSL(SectionHeader / RadioListTile)。値は `$.settings.<param>`、書き込みは settings.set。
// ---------------------------------------------------------------------------

fn section_header(key: &str, title: &str) -> Value {
    rrl::node("SectionHeader", key, json!({"title": title}), vec![])
}

/// 1 つのラジオ。`param` = パラメーター名(同じグループの全ラジオが同じ key)、`value` = 選択肢、`group` = Bind。
fn radio(param: &str, title: &str, value: &str) -> Value {
    let mut n = rrl::node(
        "RadioListTile",
        param,
        json!({"title": title, "value": value, "group": format!("$.settings.{param}")}),
        vec![],
    );
    n["on"] = json!({"change": "settings.set"});
    n
}

fn params(args: &Value) -> Result<Value, Value> {
    check_id(args)?;
    let root = rrl::column(
        "params",
        vec![
            section_header("order_header", "並び順"),
            radio(PARAM_ORDER, "昇順", "asc"),
            radio(PARAM_ORDER, "降順", "desc"),
            section_header("key_header", "並べ替えのキー"),
            radio(PARAM_KEY, "本文", "body"),
            radio(PARAM_KEY, "表示名", "name"),
            radio(PARAM_KEY, "ハンドル", "handle"),
        ],
    );
    Ok(json!({
        "root": root,
        "defaults": {PARAM_ORDER: DEFAULT_ORDER, PARAM_KEY: DEFAULT_KEY},
    }))
}

// ---------------------------------------------------------------------------
// tl.sort
// ---------------------------------------------------------------------------

fn sort_key<'a>(post: &'a Value, key: &str) -> &'a str {
    let v = match key {
        "name" => post.get("author").and_then(|a| a.get("displayName")),
        "handle" => post.get("author").and_then(|a| a.get("handle")),
        _ => post.get("body"),
    };
    v.and_then(Value::as_str).unwrap_or("")
}

fn sort(args: &Value) -> Result<Value, Value> {
    check_id(args)?;
    let params = args.get("params");
    let param = |name: &str, default: &str| -> String {
        params
            .and_then(|p| p.get(name))
            .and_then(Value::as_str)
            .unwrap_or(default)
            .to_string()
    };
    let descending = param(PARAM_ORDER, DEFAULT_ORDER) == "desc";
    let key = param(PARAM_KEY, DEFAULT_KEY);

    let posts = args.get("posts").and_then(Value::as_array);
    // (キー, id) の組。id 無しの投稿は除外(order に載らない = 表示されない)。
    let mut keyed: Vec<(&str, &str)> = posts
        .into_iter()
        .flatten()
        .filter_map(|p| Some((sort_key(p, &key), p.get("id").and_then(Value::as_str)?)))
        .collect();
    // Rust の &str 比較はバイト列比較 = UTF-8 ではコードポイント順と一致する。stable sort。
    // `desc` はキー比較だけ反転し、同点の id 昇順(tie-break)は変えない。
    keyed.sort_by(|a, b| {
        let primary = if descending { b.0.cmp(a.0) } else { a.0.cmp(b.0) };
        primary.then_with(|| a.1.cmp(b.1))
    });
    Ok(json!({ "order": keyed.iter().map(|(_, id)| *id).collect::<Vec<_>>() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(id: &str, body: &str) -> Value {
        json!({"id": id, "body": body, "createdAt": "2026-09-10T00:00:00Z"})
    }

    fn post_by(id: &str, body: &str, name: &str, handle: &str) -> Value {
        json!({"id": id, "body": body, "author": {"displayName": name, "handle": handle}, "createdAt": "2026-09-10T00:00:00Z"})
    }

    fn ids(v: Value) -> Vec<String> {
        v["order"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
    }

    fn order(posts: Vec<Value>) -> Vec<String> {
        ids(call_native("tl.sort", json!({"id": TL_UNICODE, "posts": posts})).unwrap())
    }

    fn order_with(posts: Vec<Value>, params: Value) -> Vec<String> {
        ids(call_native("tl.sort", json!({"id": TL_UNICODE, "posts": posts, "params": params})).unwrap())
    }

    /// root を深さ優先で走査し、`t` が一致するノードを集める。
    fn find_nodes<'a>(node: &'a Value, t: &str, out: &mut Vec<&'a Value>) {
        if node["t"] == json!(t) {
            out.push(node);
        }
        if let Some(c) = node.get("c").and_then(Value::as_array) {
            for ch in c {
                find_nodes(ch, t, out);
            }
        }
    }

    /// `plugin.manifest` は `manifest.json` そのもの(`rasq check` の往復検査と同じ突き合わせ)。
    #[test]
    fn manifest_matches_manifest_json() {
        let m = call_native("plugin.manifest", json!({})).unwrap();
        let file: Value = rasq_plugin_kit::serde_json::from_str(include_str!("../manifest.json")).unwrap();
        assert_eq!(m, file);
    }

    #[test]
    fn manifest_and_timelines() {
        let m = call_native("plugin.manifest", json!({})).unwrap();
        assert_eq!(m["id"], json!("sample.timeline"));
        assert_eq!(m["kind"], json!("view"));
        assert_eq!(m["slots"], json!(["tl"]));
        assert_eq!(m["permissions"], json!([]));
        let t = call_native("tl.timelines", json!({})).unwrap();
        assert_eq!(t, json!({"timelines": [{"id": "unicode-asc", "label": "Unicode 順", "base": "home"}]}));
    }

    #[test]
    fn params_has_both_radio_groups() {
        let p = call_native("tl.params", json!({"id": "unicode-asc"})).unwrap();
        assert_eq!(p["defaults"], json!({"order": "asc", "key": "body"}));
        let root = &p["root"];
        assert_eq!(root["t"], json!("Column"));

        let mut headers = vec![];
        find_nodes(root, "SectionHeader", &mut headers);
        let titles: Vec<&Value> = headers.iter().map(|h| &h["p"]["title"]).collect();
        assert_eq!(titles, [&json!("並び順"), &json!("並べ替えのキー")]);

        let mut radios = vec![];
        find_nodes(root, "RadioListTile", &mut radios);
        assert_eq!(radios.len(), 5);
        for r in &radios {
            assert_eq!(r["on"]["change"], json!("settings.set"), "radio {:?}", r);
            let k = r["k"].as_str().unwrap();
            assert_eq!(r["p"]["group"], json!(format!("$.settings.{k}")), "radio {:?}", r);
        }
        let group = |param: &str| -> Vec<(String, String)> {
            radios
                .iter()
                .filter(|r| r["k"] == json!(param))
                .map(|r| (r["p"]["value"].as_str().unwrap().to_string(), r["p"]["title"].as_str().unwrap().to_string()))
                .collect()
        };
        assert_eq!(group("order"), [("asc".into(), "昇順".into()), ("desc".into(), "降順".into())]);
        assert_eq!(
            group("key"),
            [("body".into(), "本文".into()), ("name".into(), "表示名".into()), ("handle".into(), "ハンドル".into())]
        );
        // SectionHeader → その節のラジオ、の順で並んでいる
        let kinds: Vec<&str> = root["c"].as_array().unwrap().iter().map(|n| n["t"].as_str().unwrap()).collect();
        assert_eq!(kinds, ["SectionHeader", "RadioListTile", "RadioListTile", "SectionHeader", "RadioListTile", "RadioListTile", "RadioListTile"]);

        // 未知の id は C01
        assert_eq!(call_native("tl.params", json!({"id": "nope"})).unwrap_err()["code"], json!("C01"));
        assert_eq!(call_native("tl.params", json!({})).unwrap_err()["code"], json!("C01"));
    }

    #[test]
    fn unicode_code_point_ascending() {
        // ASCII (U+0041) < ひらがな (U+3042) < カタカナ (U+30A2) < 漢字 (U+6F22) < 絵文字 (U+1F600)
        let o = order(vec![
            post("emoji", "😀"),
            post("kanji", "漢字"),
            post("kata", "ア"),
            post("hira", "あ"),
            post("ascii", "A"),
            post("digit", "1"),
        ]);
        assert_eq!(o, ["digit", "ascii", "hira", "kata", "kanji", "emoji"]);
        // 接頭辞は短い方が先
        assert_eq!(order(vec![post("ab", "ab"), post("a", "a")]), ["a", "ab"]);
    }

    #[test]
    fn missing_params_means_asc_body() {
        let posts = vec![post_by("b", "い", "Z", "@z"), post_by("a", "あ", "Y", "@y"), post_by("c", "う", "X", "@x")];
        // params 無し / 空 / 未知の値 → 本文の昇順
        assert_eq!(order(posts.clone()), ["a", "b", "c"]);
        assert_eq!(order_with(posts.clone(), json!({})), ["a", "b", "c"]);
        assert_eq!(order_with(posts.clone(), json!({"order": "sideways", "key": "mood"})), ["a", "b", "c"]);
        assert_eq!(order_with(posts, Value::Null), ["a", "b", "c"]);
    }

    #[test]
    fn descending_reverses_keys_but_not_tie_break() {
        let o = order_with(
            vec![post("emoji", "😀"), post("ascii", "A"), post("hira", "あ")],
            json!({"order": "desc"}),
        );
        assert_eq!(o, ["emoji", "hira", "ascii"]);
        // 同じキーは降順でも id 昇順のまま
        let o = order_with(
            vec![post("z", "同じ"), post("a", "同じ"), post("m", "同じ"), post("first", "😀")],
            json!({"order": "desc", "key": "body"}),
        );
        assert_eq!(o, ["first", "a", "m", "z"]);
    }

    #[test]
    fn sort_by_display_name_and_handle() {
        let posts = vec![
            post_by("p1", "う", "さくら", "@zed"),
            post_by("p2", "あ", "たろう", "@amy"),
            post_by("p3", "い", "Alice", "@mia"),
        ];
        assert_eq!(order_with(posts.clone(), json!({"key": "name"})), ["p3", "p1", "p2"]);
        assert_eq!(order_with(posts.clone(), json!({"key": "name", "order": "desc"})), ["p2", "p1", "p3"]);
        assert_eq!(order_with(posts.clone(), json!({"key": "handle"})), ["p2", "p3", "p1"]);
        assert_eq!(order_with(posts, json!({"key": "body"})), ["p2", "p3", "p1"]);
        // author 無しの投稿は空文字キー = 先頭(昇順)、同点は id 順
        let o = order_with(vec![post("noauthor2", "x"), post_by("named", "x", "B", "@b"), post("noauthor1", "x")], json!({"key": "name"}));
        assert_eq!(o, ["noauthor1", "noauthor2", "named"]);
    }

    #[test]
    fn stable_with_ties_by_id() {
        let o = order(vec![post("z", "同じ"), post("b", "同じ"), post("a", "同じ"), post("m", "同じ")]);
        assert_eq!(o, ["a", "b", "m", "z"]);
        // 同じ本文・同じ id なら入力順のまま(stable)
        let o = order(vec![post("x", "同じ"), post("x", "同じ")]);
        assert_eq!(o, ["x", "x"]);
    }

    #[test]
    fn unknown_id_is_c01() {
        let e = call_native("tl.sort", json!({"id": "nope", "posts": []})).unwrap_err();
        assert_eq!(e["code"], json!("C01"));
        let e = call_native("tl.sort", json!({"posts": []})).unwrap_err();
        assert_eq!(e["code"], json!("C01"));
        assert_eq!(call_native("tl.customize", json!({})).unwrap_err()["code"], json!("C01"));
    }

    #[test]
    fn bad_input_does_not_panic() {
        assert_eq!(call_native("tl.sort", json!({"id": "unicode-asc"})).unwrap(), json!({"order": []}));
        assert_eq!(
            call_native("tl.sort", json!({"id": "unicode-asc", "posts": [1, null, {"id": 5}, {"body": "x"}, {"id": "ok", "body": 3}]})).unwrap(),
            json!({"order": ["ok"]})
        );
        assert_eq!(
            call_native("tl.sort", json!({"id": "unicode-asc", "params": 7, "posts": [{"id": "a", "author": "str"}, {"id": "b", "author": {"handle": 1}}]})).unwrap(),
            json!({"order": ["a", "b"]})
        );
        assert!(call_native("tl.sort", Value::Null).is_err());
    }
}
