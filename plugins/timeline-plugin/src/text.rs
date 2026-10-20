//! 利用者に見せる文の表(PROTOCOL.md §4.6)。ホストの `args.locale`(無ければ `ja`)で引く。
//!
//! 基準は `ja`。`en` / `zh-Hans` / `zh-Hant` / `ko` は同じキーを全部持つ(テストで確かめる)。

rasq_plugin_kit::i18n_catalog! {
    pub(crate) static TEXT = {
        "ja": {
            "timeline.unicode" => "Unicode 順",
            "params.order" => "並び順",
            "params.order.asc" => "昇順",
            "params.order.desc" => "降順",
            "params.key" => "並べ替えのキー",
            "params.key.body" => "本文",
            "params.key.name" => "表示名",
            "params.key.handle" => "ハンドル",
            "root.desc" => "カスタム TL サンプル: 本文/表示名/ハンドルを Unicode 順に並べるソートノード「Unicode 順」",
        },
        "en": {
            "timeline.unicode" => "Unicode order",
            "params.order" => "Order",
            "params.order.asc" => "Ascending",
            "params.order.desc" => "Descending",
            "params.key" => "Sort by",
            "params.key.body" => "Text",
            "params.key.name" => "Display name",
            "params.key.handle" => "Handle",
            "root.desc" => "Custom timeline sample: “Unicode order”, a sort node that orders posts by text, display name or handle",
        },
        "zh-Hans": {
            "timeline.unicode" => "Unicode 顺序",
            "params.order" => "排序方式",
            "params.order.asc" => "升序",
            "params.order.desc" => "降序",
            "params.key" => "排序依据",
            "params.key.body" => "正文",
            "params.key.name" => "昵称",
            "params.key.handle" => "用户名",
            "root.desc" => "自定义时间线示例：按 Unicode 顺序排列正文、昵称或用户名的排序节点「Unicode 顺序」",
        },
        "zh-Hant": {
            "timeline.unicode" => "Unicode 順序",
            "params.order" => "排序方式",
            "params.order.asc" => "遞增",
            "params.order.desc" => "遞減",
            "params.key" => "排序依據",
            "params.key.body" => "內文",
            "params.key.name" => "顯示名稱",
            "params.key.handle" => "使用者名稱",
            "root.desc" => "自訂時間軸範例：依 Unicode 順序排列內文、顯示名稱或使用者名稱的排序節點「Unicode 順序」",
        },
        "ko": {
            "timeline.unicode" => "Unicode 순",
            "params.order" => "정렬 순서",
            "params.order.asc" => "오름차순",
            "params.order.desc" => "내림차순",
            "params.key" => "정렬 기준",
            "params.key.body" => "본문",
            "params.key.name" => "표시 이름",
            "params.key.handle" => "핸들",
            "root.desc" => "커스텀 타임라인 샘플: 본문·표시 이름·핸들을 Unicode 순으로 정렬하는 정렬 노드 ‘Unicode 순’",
        },
    };
}

#[cfg(test)]
mod tests {
    use super::TEXT;
    use std::collections::BTreeSet;

    /// この表の言語ごとのキー(`Catalog::check` は「基準に無いキー」と差し込みの食い違いしか見ないので、訳の抜けはここで見る)。
    fn table_keys() -> Vec<(String, BTreeSet<String>)> {
        let mut out: Vec<(String, BTreeSet<String>)> = Vec::new();
        for line in include_str!("text.rs").lines().map(str::trim) {
            if let Some(tag) = line.strip_prefix('"').and_then(|l| l.strip_suffix("\": {")) {
                out.push((tag.to_string(), BTreeSet::new()));
            } else if let Some((key, _)) = line.strip_prefix('"').and_then(|l| l.split_once("\" =>")) {
                out.last_mut().expect("表の外のキー").1.insert(key.to_string());
            }
        }
        out
    }

    #[test]
    fn every_locale_has_every_key() {
        assert!(TEXT.check().is_empty(), "{:?}", TEXT.check());
        let tables = table_keys();
        let tags: Vec<&str> = tables.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(tags, TEXT.locales());
        assert_eq!(tags, ["ja", "en", "zh-Hans", "zh-Hant", "ko"]);
        for (tag, keys) in &tables[1..] {
            let missing: Vec<_> = tables[0].1.difference(keys).collect();
            assert!(missing.is_empty(), "{tag} に訳が無いキー: {missing:?}");
        }
    }
}
