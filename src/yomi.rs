use std::{
    io::Write,
    ops::{Not, Range},
};

use crate::{error::OrDie, 死};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yomi<'src> {
    pub span: Range<usize>,
    pub rt: &'src str,
}

pub fn write_yomi(yomi: &[Yomi], mut file: impl Write, txt: &str) {
    let mut buf = String::new();
    for &Yomi {
        span: Range { start, end },
        rt,
        ..
    } in yomi
    {
        let rb = &txt[start..end];
        let rt = fix_little_yomi(rb, rt, &mut buf);
        writeln!(file, "{start}:{end}:{rb}:{rt}").or_(死!());
    }
}

fn fix_little_yomi<'s>(rb: &str, rt: &'s str, fixed: &'s mut String) -> &'s str {
    let Some(little_idx) = rt.find(['や', 'ゆ', 'よ']) else {
        return rt;
    };
    if little_idx < 3 {
        return rt;
    }
    let previous = little_idx - 3;

    // "お や じ" "み や こ" etc.
    if rt.is_char_boundary(previous).not() {
        fixed.clear();
        fixed.extend(rt.chars().filter(|&ch| ch != ' '));
        return fixed.as_str();
    }
    match &rt[previous..little_idx] {
        "き" | "ぎ" | "し" | "じ" | "ち" | "ぢ" | "に" | "ひ" | "び" | "ぴ" | "み" | "り" =>
            {}
        _ => return rt,
    }

    if ["清", "日和"].contains(&rb) {
        return rt;
    }

    let next = little_idx + 3;
    debug_assert!(rt.is_char_boundary(next));

    fixed.clear();
    fixed.push_str(&rt[..little_idx]);
    match &rt[little_idx..next] {
        "や" => fixed.push('ゃ'),
        "ゆ" => fixed.push('ゅ'),
        "よ" => fixed.push('ょ'),
        _ => unreachable!(),
    }
    fixed.push_str(&rt[next..]);
    fixed.as_str()
}

#[test]
fn test_fix_little_yomi() {
    let mut fixed = String::new();
    for (rb, rt, expected) in [
        ("日和", "びより", "びより"),
        ("清", "きよ", "きよ"),
        ("喋", "しやべ", "しゃべ"),
        ("華", "きや", "きゃ"),
        ("奢", "しや", "しゃ"),
        ("椒", "しよう", "しょう"),
        ("弱", "じやく", "じゃく"),
        ("手", "しゆ", "しゅ"),
        ("榴", "りゆう", "りゅう"),
        ("百", "ぴやく", "ぴゃく"),
        ("焼", "しよう", "しょう"),
        ("榴", "りゆう", "りゅう"),
        ("車", "しや", "しゃ"),
        ("驚", "きよう", "きょう"),
        ("榴", "りゆう", "りゅう"),
        ("嬌", "きよう", "きょう"),
        ("怯", "きよう", "きょう"),
        ("厨", "ちゆう", "ちゅう"),
        ("頭", "じゆう", "じゅう"),
        ("薯", "じよ", "じょ"),
        ("玩具", "おもちや", "おもちゃ"),
        ("親父", "お や じ", "おやじ"),
    ] {
        assert_eq!(fix_little_yomi(rb, rt, &mut fixed), expected);
    }
}
