//! timeline-plugin — カスタム TL(`tl`)のサンプル。
//!
//! ホーム TL に「Unicode 昇順」タブを足し、本文を Unicode コードポイント昇順に並べ替える。
//! データ取得はホストの仕事で、このプラグインは宣言(`tl.timelines`)と並べ替え(`tl.sort`)だけ。
use rasq_plugin_kit::{err, manifest, rasq_plugin, rrl, serde_json::{json, Value}, unsupported};

const ID: &str = "sample.timeline";
const TL_UNICODE_ASC: &str = "unicode-asc";

fn root() -> Value {
    rrl::column("root", vec![rrl::text("desc", "カスタム TL サンプル: 本文を Unicode 昇順で並べる「Unicode 昇順」タブ")])
}

fn handle(fn_name: &str, args: Value) -> Result<Value, Value> {
    match fn_name {
        "plugin.manifest" => Ok(manifest(
            ID,
            "Unicode 昇順 TL(サンプル)",
            "0.1.0",
            "view",
            "本文を Unicode コードポイント昇順で並べるカスタムタイムライン",
            &["tl"],
            &[],
        )),
        "lifecycle" => Ok(json!({})),
        "tl.timelines" => Ok(json!({
            "timelines": [{"id": TL_UNICODE_ASC, "label": "Unicode 昇順", "base": "home"}]
        })),
        "tl.sort" => sort(&args),
        _ => Err(unsupported(fn_name)),
    }
}

rasq_plugin! { scope: "sample.timeline:main", root: root, call: handle }

fn sort(args: &Value) -> Result<Value, Value> {
    let id = args.get("id").and_then(Value::as_str).unwrap_or("");
    if id != TL_UNICODE_ASC {
        return Err(err("C01", format!("unknown timeline: {id:?}")));
    }
    let posts = args.get("posts").and_then(Value::as_array);
    // (body, id) の組。id 無しの投稿は除外(order に載らない = 表示されない)。
    let mut keyed: Vec<(&str, &str)> = posts
        .into_iter()
        .flatten()
        .filter_map(|p| {
            let id = p.get("id").and_then(Value::as_str)?;
            Some((p.get("body").and_then(Value::as_str).unwrap_or(""), id))
        })
        .collect();
    // Rust の &str 比較はバイト列比較 = UTF-8 ではコードポイント順と一致する。stable sort。
    keyed.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(b.1)));
    Ok(json!({ "order": keyed.iter().map(|(_, id)| *id).collect::<Vec<_>>() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(id: &str, body: &str) -> Value {
        json!({"id": id, "body": body, "createdAt": "2026-09-10T00:00:00Z"})
    }

    fn order(posts: Vec<Value>) -> Vec<String> {
        call_native("tl.sort", json!({"id": TL_UNICODE_ASC, "posts": posts}))
            .unwrap()["order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn manifest_and_timelines() {
        let m = call_native("plugin.manifest", json!({})).unwrap();
        assert_eq!(m["id"], json!("sample.timeline"));
        assert_eq!(m["kind"], json!("view"));
        assert_eq!(m["slots"], json!(["tl"]));
        assert_eq!(m["permissions"], json!([]));
        let t = call_native("tl.timelines", json!({})).unwrap();
        assert_eq!(t, json!({"timelines": [{"id": "unicode-asc", "label": "Unicode 昇順", "base": "home"}]}));
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
        assert!(call_native("tl.sort", Value::Null).is_err());
    }
}
