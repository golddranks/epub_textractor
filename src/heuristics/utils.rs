use std::collections::HashMap;

use crate::{
    chapters::Role,
    epub::Epub,
    error::{OrDie, 即死, 死},
};

pub fn contains_any_of(name: &str, words: &[&str]) -> bool {
    for word in words {
        if name.contains(word) {
            return true;
        }
    }
    false
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

pub fn contains_numerals(s: &str) -> bool {
    s.chars()
        .map(convert_kanji_numerals)
        .map(convert_zenkaku)
        .find(|c| c.is_digit(10))
        .is_some()
}

pub fn convert_kanji_numerals(c: char) -> char {
    match c {
        '零' => '0',
        '一' | '壱' => '1',
        '二' | '弍' => '2',
        '三' | '参' => '3',
        '四' => '4',
        '五' | '伍' => '5',
        '六' | '陸' => '6',
        '七' | '漆' | '質' => '7',
        '八' | '捌' => '8',
        '九' | '玖' => '9',
        '十' | '拾' | '什' => '0',
        _ => c,
    }
}

pub fn convert_zenkaku(c: char) -> char {
    match c {
        // FF5E (FULLWIDTH TILDE, '～') is VERY easy to mix up with 301C (WAVE DASH, '〜')
        // macOS key layouts emit WAVE DASH by default as fullwidth tilde,
        // so it is not rare in the wild – so let's handle it too.
        '！'..='\u{FF5E}' => ((c as u32 - '！' as u32) as u8 + b'!') as char,
        '\u{301C}' => '~',
        '\u{3000}' => ' ',
        _ => c,
    }
}

#[test]
fn test_convert_zenkaku() {
    assert_eq!(
        "ＡＢＣ！１２３\u{FF5E}"
            .chars()
            .map(convert_zenkaku)
            .collect::<String>(),
        "ABC!123~"
    );
    assert_eq!(
        "ＡＢＣ！１２３\u{301C}"
            .chars()
            .map(convert_zenkaku)
            .collect::<String>(),
        "ABC!123~"
    );
}

pub fn n_books(epub: &Epub) -> usize {
    let title = &epub.title;
    if title.contains("合本版") || title.contains("セット") {
        let start_idx = title.find("全").or_(死!("No 全 found in title")) + "全".len();
        let end_idx = title[start_idx..]
            .find('巻')
            .or_(死!("No 巻 found in title"))
            + start_idx;
        title[start_idx..end_idx]
            .chars()
            .map(convert_zenkaku)
            .collect::<String>()
            .parse()
            .or_(死!("Failed to parse number"))
    } else {
        1
    }
}

/// A fault tolerant way to get the spine index by href.
/// (Some EPUBS are buggy; they don't have everything in manifest.)
pub fn get_spine_idx(
    href_to_spine_idx: &HashMap<String, usize>,
    toc_href: &str,
    name: &str,
) -> usize {
    if let Some(spine_idx) = href_to_spine_idx.get(toc_href) {
        *spine_idx
    } else if name == "表紙" {
        0 // for some reason, many buggy EPUBS don't have the title page in their manifest? shame on them.
    } else {
        即死!("no manifest href that corresponds to the TOC href {toc_href}? ({name})")
    }
}
