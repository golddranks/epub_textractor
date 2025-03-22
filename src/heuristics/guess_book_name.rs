use crate::epub::Epub;

pub fn guess_book_name(epub: &Epub) -> String {
    // Leaving spaces to corners to be able to catch some words separated only by spaces
    let mut title = format!(" {} ", epub.title);
    let mut remove = |start: &str, mid: &str, end: &str| {
        let mut pos = 0;
        // start searching from middle part for non-greedyness
        while let Some(mut mid_idx) = title[pos..].find(mid) {
            mid_idx += pos;
            let start_idx = title[..mid_idx].rfind(start);
            let end_idx = title[mid_idx..].find(end);
            if let (Some(start_idx), Some(mut end_idx)) = (start_idx, end_idx) {
                end_idx += mid_idx;
                let final_idx = end_idx + end.len();
                pos = start_idx;
                title.replace_range(start_idx..final_idx, "");
                continue;
            }
            // we didn't manage to replace anything, but there was at least some progress
            // next time, start at least past mid_idx
            pos = mid_idx + mid.len();
        }
    };

    remove("【", "版", "】");
    remove("【", "付", "】");
    remove("【", "入", "】");
    remove("【", "セット", "】");
    remove("【", "シリーズ", "】");
    remove("【", "小説", "】");
    remove("［", "版", "］");
    remove("〈", "版", "〉");
    remove("(", "文庫", ")");
    remove("（", "文庫", "）");
    remove("(", "ノベル", ")");
    remove("（", "ノベル", "）");
    remove("(", "ブックス", ")");
    remove("(", "BOOKS", ")");
    remove("(", "NOVELS", ")");
    remove("(", "書庫", ")");
    remove("(", "小説", ")");
    remove("(", "書店", ")");
    remove("(", "キス", ")");
    remove("(", "ファンタジー", ")");
    remove("(", "社", ")");
    remove("(", "版", ")");
    remove("（", "版", "）");
    remove("(", "文芸", ")");
    remove(" ", "文庫", " ");
    remove(" ", "シリーズ", " ");

    for word in [
        "新装版",
        "(幅広)",
        "(Kindle Single)",
        "(アイリスNEO)",
        "(サーガフォレスト)",
        "（サーガフォレスト）",
        "(アース・スター ルナ)",
    ] {
        title = title.replace(word, "");
    }

    title.trim().to_owned()
}

#[test]
fn test_guess_book_name() {
    let mut epub = Epub {
        title: "転生したらばかだった【SS付き電子限定版】(hogeブックス)".to_owned(),
        author: "hoge".to_owned(),
        publisher: String::new(),
        body: vec![],
        href_to_spine_idx: std::collections::HashMap::new(),
        toc: vec![],
    };
    assert_eq!(guess_book_name(&epub), "転生したらばかだった");

    epub.title = "転生したらばかだった(2)【SS付き電子限定版】(hogeブックス)".to_owned();
    assert_eq!(guess_book_name(&epub), "転生したらばかだった(2)");

    epub.title = "転生したらばかだった(3)【SS付き】【イラスト付き】".to_owned();
    assert_eq!(guess_book_name(&epub), "転生したらばかだった(3)");
}
