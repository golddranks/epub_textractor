use std::collections::HashMap;

use crate::{
    chapters::{self, Role},
    epub::Epub,
    error::{OrDie, 即死},
    死,
};

fn contains_any_of(name: &str, words: &[&str]) -> bool {
    for word in words {
        if name.contains(word) {
            return true;
        }
    }
    false
}

fn is_toc(name: &str) -> bool {
    contains_any_of(
        name,
        &[
            "目次",
            "もくじ",
            "ＣＯＮＴＥＮＴＳ",
            "contents",
            "Contents",
            "CONTENTS",
            "Ｍｅｎｕ",
        ],
    )
}

fn is_before_extra(name: &str) -> bool {
    contains_any_of(name, &["紹介", "登場人物"])
}

fn is_afterword(name: &str) -> bool {
    contains_any_of(name, &["あとがき", "後書"])
}

fn is_prologue(name: &str) -> bool {
    contains_any_of(name, &["プロローグ"])
}

fn is_epilogue(name: &str) -> bool {
    contains_any_of(name, &["エピローグ"])
}

fn is_cover(name: &str) -> bool {
    contains_any_of(name, &["表紙", "表題紙"])
}

fn is_copyright(name: &str) -> bool {
    contains_any_of(name, &["奥付"])
}

fn anything_goes(_: &str) -> bool {
    true
}

fn assumed_order(r: Role) -> usize {
    match r {
        Role::Cover => 0,
        Role::BeforeExtra => 1,
        Role::Foreword => 1,
        Role::Contents => 1,
        Role::Prologue => 2,
        Role::Main => 3,
        Role::Epilogue => 4,
        Role::BonusChapter => 5,
        Role::Afterword => 6,
        Role::AfterExtra => 6,
        Role::Copyright => 7,
    }
}

pub fn guess_role(chapters: &[chapters::Chapter], name: &str) -> Role {
    let highest = chapters.last().map(|c| c.role).unwrap_or(Role::Cover);
    let tests = [
        (true, is_cover as fn(&str) -> bool, Role::Cover),
        (true, is_before_extra, Role::BeforeExtra),
        (true, is_toc, Role::Contents),
        (true, is_prologue, Role::Prologue),
        (true, is_epilogue, Role::Epilogue),
        (true, is_afterword, Role::Afterword),
        (true, is_copyright, Role::Copyright),
        (false, anything_goes, Role::Foreword),
        (false, anything_goes, Role::Main),
        (false, anything_goes, Role::BonusChapter),
        (false, anything_goes, Role::AfterExtra),
    ];
    for (reliable, test, role) in tests {
        let matches = test(name);
        if assumed_order(role) < assumed_order(highest) {
            if reliable && matches {
                即死!(
                    "Chapter {name} is out of order with role {}: {:?}",
                    role,
                    chapters
                );
            }
            continue;
        }
        if matches {
            return role;
        }
    }
    即死!("No role found for chapter: {name}");
}

pub fn is_skip(role: Role) -> bool {
    match role {
        Role::Cover
        | Role::BeforeExtra
        | Role::Foreword
        | Role::Contents
        | Role::Afterword
        | Role::AfterExtra
        | Role::Copyright => true,
        Role::Prologue | Role::Main | Role::Epilogue | Role::BonusChapter => false,
    }
}

fn convert_zenkaku_number(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '０'..='９' => ((c as u32 - '０' as u32) as u8 + b'0') as char,
            _ => c,
        })
        .collect()
}

pub fn n_books(epub: &Epub) -> usize {
    let title = &epub.title;
    if title.contains("合本版") || title.contains("セット") {
        let start_idx = title.find("全").or_(死!("No 全 found in title")) + "全".len();
        let end_idx = title[start_idx..]
            .find('巻')
            .or_(死!("No 巻 found in title"))
            + start_idx;
        convert_zenkaku_number(&title[start_idx..end_idx])
            .parse()
            .or_(死!("Failed to parse number"))
    } else {
        1
    }
}

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

/// A fault tolerant way to get the spine index by href.
/// (Some EPUBS are buggy; they don't have everything in manifest.)
pub fn get_spine_idx(href_to_spine_idx: &HashMap<String, usize>, toc_href: &str, name: &str) -> usize {
    if let Some(spine_idx) = href_to_spine_idx.get(toc_href) {
        *spine_idx
    } else if name == "表紙" {
        0 // for some reason, many buggy EPUBS don't have the title page in their manifest? shame on them.
    } else {
        即死!("no manifest href that corresponds to the TOC href {toc_href}? ({name})")
    }
}
