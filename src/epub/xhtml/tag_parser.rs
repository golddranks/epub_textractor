use std::ops::{Not, Range};

use crate::Res;

use super::{TType, Tag, XhtmlError};

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

fn parse_quotes(source: &str) -> Res<Range<usize>> {
    let start = source.find(['"', '\'']).ok_or(XhtmlError::UnexpectedEOF)?;
    let quotation_mark = source.as_bytes()[start];
    let mut pos = start + 1;
    while source[pos..].is_empty().not() {
        pos += source[pos..]
            .find(quotation_mark as char)
            .ok_or(XhtmlError::UnexpectedEOF)?;
        if source.as_bytes()[pos - 1] != b'\\' {
            return Ok(start..pos + 1);
        } else {
            pos += 1;
        }
    }
    Err(XhtmlError::UnexpectedEOF)?
}

#[test]
fn test_parse_quotes() -> Res<()> {
    assert!(parse_quotes(r#""#).is_err());
    assert!(parse_quotes(r#"""#).is_err());
    assert_eq!(parse_quotes(r#""""#)?, 0..2);
    assert_eq!(parse_quotes(r#"a"b"c"#)?, 1..4);
    assert_eq!(parse_quotes(r#"''"#)?, 0..2);
    assert_eq!(parse_quotes(r#"a'b'c"#)?, 1..4);
    assert!(parse_quotes(r#"a'b"c"#).is_err());
    assert!(parse_quotes(r#"a"b'c"#).is_err());
    assert_eq!(parse_quotes(r#"a"b\""c"#)?, 1..6);
    assert_eq!(parse_quotes(r#"a"あ"c"#)?, 1..6);
    assert_eq!(parse_quotes(r#""fuga">noniin"#)?, 0..6);
    Ok(())
}

pub fn parse_tag(source: &str, offset: usize) -> Res<Option<Tag>> {
    // find starting <
    let Some(start) = source[offset..].find('<').map(|s| offset + s) else {
        return Ok(None);
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
        .ok_or(XhtmlError::MalformedTagName)?;
    let tag_name = &source[pos..pos + tag_name_end];
    pos += tag_name_end;

    // scan until end, while minding quotes in attributes
    loop {
        pos += source[pos..]
            .find(['>', '"', '\''])
            .ok_or(XhtmlError::CannotFindTagEnd)?;
        if source.as_bytes()[pos] == b'>' {
            break;
        }
        let quote = parse_quotes(&source[pos..])?;
        pos += quote.end;
    }

    // check for self-closing tag marker
    let self_closing_tag = source.as_bytes()[pos - 1] == b'/';
    pos += 1;

    Ok(Some(Tag {
        name: tag_name,
        source,
        span: start..pos,
        before_text: &source[offset..start],
        kind: match (closing_tag, self_closing_tag) {
            (false, false) => TType::Opening,
            (true, false) => TType::Closing,
            (false, true) => TType::SelfClosing,
            (true, true) => Err(XhtmlError::MixedClosingMarks)?,
        },
    }))
}

#[test]
fn test_parse_tag() {
    assert_eq!(parse_tag("<hoge>", 0).unwrap().unwrap().span, 0..6);
    assert_eq!(parse_tag("<hoge/>", 0).unwrap().unwrap().span, 0..7);
    assert_eq!(parse_tag("<hoge />", 0).unwrap().unwrap().span, 0..8);
    assert_eq!(parse_tag("<hoge>", 0).unwrap().unwrap().name, "hoge");
    assert_eq!(parse_tag("<hoge/>", 0).unwrap().unwrap().name, "hoge");
    assert_eq!(parse_tag("<hoge />", 0).unwrap().unwrap().name, "hoge");
    assert_eq!(
        parse_tag("<hoge>after hoge", 0).unwrap().unwrap().span,
        0..6
    );
    assert_eq!(
        parse_tag(r#"<hoge param="fuga">noniin"#, 0)
            .unwrap()
            .unwrap()
            .span,
        0..19
    );
    assert_eq!(
        parse_tag(r#"<hoge param="fu>ga">noniin"#, 0)
            .unwrap()
            .unwrap()
            .span,
        0..20
    );
    assert_eq!(
        parse_tag(r#"<hoge param="fu\"ga">juu"#, 0)
            .unwrap()
            .unwrap()
            .span,
        0..21
    );
    assert_eq!(
        parse_tag(r#"<hoge param="あ">juu"#, 0)
            .unwrap()
            .unwrap()
            .span,
        0..18
    );

    let self_closing = parse_tag(r#"<hoge param="fuga" />jooh"#, 0)
        .unwrap()
        .unwrap();
    assert_eq!(self_closing.span, 0..21);
    assert_eq!(self_closing.kind, TType::SelfClosing);

    let closing = parse_tag("</hoge>juuh", 0).unwrap().unwrap();
    assert_eq!(closing.span, 0..7);
    assert_eq!(closing.kind, TType::Closing);

    let あ = parse_tag(r#"<あ>juu"#, 0).unwrap().unwrap();
    assert_eq!(あ.span, 0..5);
}

pub fn parse_attr<'src>(source: &'src str, target_attr: &str) -> Res<Option<&'src str>> {
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
            let mut span = parse_quotes(&source[pos..])?;
            span.start += pos;
            span.end += pos;
            pos = span.end;
            &source[span.start + 1..span.end - 1]
        } else {
            attr_name
        };
        if attr_name == target_attr {
            return Ok(Some(attr_val));
        }
    }
    Ok(None)
}

#[test]
fn test_parse_attr() {
    let source = r#"<hoge bb="cc" dd="ee" ff gg='hh'>"#;
    assert_eq!(parse_attr(source, "hoge").unwrap(), None);
    assert_eq!(parse_attr(source, "bb").unwrap(), Some("cc"));
    assert_eq!(parse_attr(source, "cc").unwrap(), None);
    assert_eq!(parse_attr(source, "dd").unwrap(), Some("ee"));
    assert_eq!(parse_attr(source, "ee").unwrap(), None);
    assert_eq!(parse_attr(source, "ff").unwrap(), Some("ff"));
    assert_eq!(parse_attr(source, "gg").unwrap(), Some("hh"));
}
