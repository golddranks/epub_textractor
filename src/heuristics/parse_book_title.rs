use crate::error::即死;

fn remove(title: &mut String, removed: &mut Vec<String>, start: &str, mid: &str, end: &str) {
    let mut pos = 0;
    // start searching from middle part for non-greedyness
    while let Some(mut mid_idx) = title[pos..].find(mid) {
        mid_idx += pos;
        let start_idx = title[..mid_idx].rfind(start);
        let end_idx = title[mid_idx..].find(end);
        if let (Some(start_idx), Some(mut end_idx)) = (start_idx, end_idx) {
            end_idx += mid_idx;
            removed.push(title[start_idx + start.len()..end_idx].to_owned());
            let final_idx = end_idx + end.len();
            pos = start_idx;
            title.replace_range(start_idx..final_idx, "");
            continue;
        }
        // we didn't manage to replace anything, but there was at least some progress
        // next time, start at least past mid_idx
        pos = mid_idx + mid.len();
    }
}

pub fn parse_book_title(title: &str) -> (String, Option<String>) {
    // Leaving spaces to corners to be able to catch some words separated only by spaces
    let mut title = format!(" {} ", title);

    let notes = [
        ("【", "版", "】"),
        ("【", "付", "】"),
        ("【", "入", "】"),
        ("【", "セット", "】"),
        ("【", "シリーズ", "】"),
        ("【", "小説", "】"),
        ("［", "版", "］"),
        ("〈", "版", "〉"),
        ("(", "版", ")"),
        ("（", "版", "）"),
        (" ", "シリーズ", " "),
    ];

    for (start, mid, end) in notes {
        remove(&mut title, &mut vec![], start, mid, end);
    }

    for word in ["新装版", "(幅広)"] {
        title = title.replace(word, "");
    }

    let labels = [
        ("(", "文庫", ")"),
        ("（", "文庫", "）"),
        ("(", "ノベル", ")"),
        ("（", "ノベル", "）"),
        ("(", "ブックス", ")"),
        ("(", "BOOKS", ")"),
        ("(", "NOVELS", ")"),
        ("(", "書庫", ")"),
        ("(", "小説", ")"),
        ("(", "書店", ")"),
        ("(", "キス", ")"),
        ("(", "ファンタジー", ")"),
        ("(", "社", ")"),
        ("(", "文芸", ")"),
        (" ", "文庫", " "),
        ("(", "Kindle Single", ")"),
        ("(", "アイリスNEO", ")"),
        ("(", "サーガフォレスト", ")"),
        ("（", "サーガフォレスト", "）"),
        ("(", "アース・スター ルナ", ")"),
    ];

    let mut label = Vec::new();
    for (start, mid, end) in labels {
        remove(&mut title, &mut label, start, mid, end);
    }

    match label.len() {
        0 | 1 => (title.trim().to_owned(), label.pop()),
        _ => 即死!("More than one label?"),
    }
}

#[test]
fn test_guess_book_name() {
    let unparsed_title = "転生したらばかだった【SS付き電子限定版】(hogeブックス)";
    let (title, label) = parse_book_title(unparsed_title);
    assert_eq!(title, "転生したらばかだった");
    assert_eq!(label.unwrap(), "hogeブックス");

    let unparsed_title = "転生したらばかだった(2)【SS付き電子限定版】(hogeブックス)";
    let (title, label) = parse_book_title(unparsed_title);
    assert_eq!(title, "転生したらばかだった(2)");
    assert_eq!(label.unwrap(), "hogeブックス");

    let unparsed_title = "転生したらばかだった(3)【SS付き】【イラスト付き】";
    let (title, label) = parse_book_title(unparsed_title);
    assert_eq!(title, "転生したらばかだった(3)");
    assert_eq!(label, None);
}
