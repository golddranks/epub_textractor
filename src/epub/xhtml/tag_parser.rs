use std::ops::{Not, Range};

use crate::error::{OrDie, 即死, 死};

use super::{TType, Tag};

fn is_attr_name_char(ch: u8) -> bool {
    ![b' ', b'\t', b'\n', b'\r', b'=', b'>', b'/', b'\'', b'"'].contains(&ch)
}

fn is_whitespace(ch: u8) -> bool {
    [b' ', b'\t', b'\n', b'\r'].contains(&ch)
}

fn consume_while(source: &str, predicate: impl Fn(u8) -> bool) -> usize {
    let mut pos = 0;
    while predicate(source.as_bytes()[pos]) {
        pos += 1;
    }
    pos
}

fn parse_quotes(source: &str) -> Range<usize> {
    let start = source.find(['"', '\'']).or_(死!());
    let quotation_mark = source.as_bytes()[start];
    let mut pos = start + 1;
    while source[pos..].is_empty().not() {
        pos += source[pos..].find(quotation_mark as char).or_(死!());
        if source.as_bytes()[pos - 1] != b'\\' {
            return start..pos + 1;
        } else {
            pos += 1;
        }
    }
    即死!("unexpected EOF")
}

#[test]
fn test_parse_quotes() {
    //assert!(parse_quotes(r#""#));
    //assert!(parse_quotes(r#"""#));
    assert_eq!(parse_quotes(r#""""#), 0..2);
    assert_eq!(parse_quotes(r#"a"b"c"#), 1..4);
    assert_eq!(parse_quotes(r#"''"#), 0..2);
    assert_eq!(parse_quotes(r#"a'b'c"#), 1..4);
    //assert!(parse_quotes(r#"a'b"c"#));
    //assert!(parse_quotes(r#"a"b'c"#));
    assert_eq!(parse_quotes(r#"a"b\""c"#), 1..6);
    assert_eq!(parse_quotes(r#"a"あ"c"#), 1..6);
    assert_eq!(parse_quotes(r#""fuga">noniin"#), 0..6);
}

pub fn parse_tag(source: &str, offset: usize) -> Option<Tag> {
    // find starting <
    let Some(start) = source[offset..].find('<').map(|s| offset + s) else {
        return None;
    };
    let mut pos = start + 1;

    // check for closing tag marker
    let closing_tag = source[pos..].as_bytes()[0] == b'/';
    if closing_tag {
        pos += 1;
    }

    // parse tag name
    let tag_name_end = source[pos..]
        .find([' ', '/', '\t', '\n', '\r', '>'])
        .or_(死!("malformed tag name"));
    let tag_name = &source[pos..pos + tag_name_end];
    pos += tag_name_end;

    // scan until end, while minding quotes in attributes
    loop {
        pos += source[pos..]
            .find(['>', '"', '\''])
            .or_(死!("cannot find tag end"));
        if source.as_bytes()[pos] == b'>' {
            break;
        }
        let quote = parse_quotes(&source[pos..]);
        pos += quote.end;
    }

    // check for self-closing tag marker
    let self_closing_tag = source.as_bytes()[pos - 1] == b'/';
    pos += 1;

    Some(Tag {
        name: tag_name,
        source,
        span: start..pos,
        before_text: &source[offset..start],
        kind: match (closing_tag, self_closing_tag) {
            (false, false) => TType::Opening,
            (true, false) => TType::Closing,
            (false, true) => TType::SelfClosing,
            (true, true) => 即死!("mixed closing marks"),
        },
    })
}

#[test]
fn test_parse_tag() {
    assert_eq!(parse_tag("<hoge>", 0).unwrap().span, 0..6);
    assert_eq!(parse_tag("<hoge/>", 0).unwrap().span, 0..7);
    assert_eq!(parse_tag("<hoge />", 0).unwrap().span, 0..8);
    assert_eq!(parse_tag("<hoge>", 0).unwrap().name, "hoge");
    assert_eq!(parse_tag("<hoge/>", 0).unwrap().name, "hoge");
    assert_eq!(parse_tag("<hoge />", 0).unwrap().name, "hoge");
    assert_eq!(parse_tag("<hoge>after hoge", 0).unwrap().span, 0..6);
    assert_eq!(
        parse_tag(r#"<hoge param="fuga">noniin"#, 0).unwrap().span,
        0..19
    );
    assert_eq!(
        parse_tag(r#"<hoge param="fu>ga">noniin"#, 0).unwrap().span,
        0..20
    );
    assert_eq!(
        parse_tag(r#"<hoge param="fu\"ga">juu"#, 0).unwrap().span,
        0..21
    );
    assert_eq!(parse_tag(r#"<hoge param="あ">juu"#, 0).unwrap().span, 0..18);

    let self_closing = parse_tag(r#"<hoge param="fuga" />jooh"#, 0).unwrap();
    assert_eq!(self_closing.span, 0..21);
    assert_eq!(self_closing.kind, TType::SelfClosing);

    let closing = parse_tag("</hoge>juuh", 0).unwrap();
    assert_eq!(closing.span, 0..7);
    assert_eq!(closing.kind, TType::Closing);

    let あ = parse_tag(r#"<あ>juu"#, 0).unwrap();
    assert_eq!(あ.span, 0..5);
}

pub fn parse_attr<'src>(source: &'src str, target_attr: &str) -> Option<&'src str> {
    let source = &source[1..source.len() - 1]; // remove < and >
    let mut pos = 0;
    pos += consume_while(&source[pos..], is_attr_name_char);
    while source[pos..].is_empty().not() {
        pos += consume_while(&source[pos..], is_whitespace);
        let attr_name_end = consume_while(&source[pos..], is_attr_name_char);
        let attr_name = &source[pos..pos + attr_name_end];
        pos += attr_name_end;
        pos += consume_while(&source[pos..], is_whitespace);
        let attr_val = if source.as_bytes()[pos] == b'=' {
            pos += 1;
            let mut span = parse_quotes(&source[pos..]);
            span.start += pos;
            span.end += pos;
            pos = span.end;
            &source[span.start + 1..span.end - 1]
        } else {
            attr_name
        };
        if attr_name == target_attr {
            return Some(attr_val);
        }
    }
    None
}

#[test]
fn test_parse_attr() {
    let source = r#"<hoge bb="cc" dd="ee" ff gg='hh'>"#;
    assert_eq!(parse_attr(source, "hoge"), None);
    assert_eq!(parse_attr(source, "bb"), Some("cc"));
    assert_eq!(parse_attr(source, "cc"), None);
    assert_eq!(parse_attr(source, "dd"), Some("ee"));
    assert_eq!(parse_attr(source, "ee"), None);
    assert_eq!(parse_attr(source, "ff"), Some("ff"));
    assert_eq!(parse_attr(source, "gg"), Some("hh"));
}
