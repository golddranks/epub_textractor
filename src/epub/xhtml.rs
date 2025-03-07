use std::{error::Error, fmt::Display, ops::Range};

use super::Res;

pub mod iter;
mod tag_parser;

use iter::TagIter;
use tag_parser::parse_attr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum XhtmlError {
    MalformedTagName,
    CannotFindTagEnd,
    MixedClosingMarks,
    ClosingTagMismatch,
    UnexpectedEOF,
    UnexpectedNotFound,
}

impl Display for XhtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XhtmlError::MalformedTagName => f.write_str("Invalid XHTML: MalformedTagName!"),
            XhtmlError::CannotFindTagEnd => f.write_str("Invalid XHTML: CannotFindTagEnd!"),
            XhtmlError::MixedClosingMarks => f.write_str("Invalid XHTML: MixedClosingMarks!"),
            XhtmlError::ClosingTagMismatch => f.write_str("Invalid XHTML: ClosingTagMismatch!"),
            XhtmlError::UnexpectedEOF => f.write_str("Invalid XHTML: Unexpected EOF!"),
            XhtmlError::UnexpectedNotFound => f.write_str("Unexpected: Tag not found!"),
        }
    }
}

impl Error for XhtmlError {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TType {
    Opening,
    Closing,
    SelfClosing,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tag<'src> {
    pub name: &'src str, // TODO: refactor to usize + accessor method?
    source: &'src str,
    span: Range<usize>,
    pub before_text: &'src str, // refactor to usize + accessor method
    pub kind: TType,
}

impl<'src> Tag<'src> {
    pub fn get_first(source: &'src str, tag: &str) -> Res<Option<Self>> {
        Tag::root(source).iter()?.next_by_el(&[tag])
    }

    pub fn get_first_child(&self, tag: &str) -> Res<Option<Self>> {
        self.iter()?.next_by_el(&[tag])
    }

    pub fn get_end(&self) -> Res<(Tag<'src>, &'src str)> {
        Ok(self
            .iter()?
            .step_out(self)?
            .unwrap_or_else(|| (self.to_owned(), "")))
    }

    pub fn get_attr(&self, target_attr: &'src str) -> Res<Option<&'src str>> {
        parse_attr(self.repr(), target_attr)
    }

    pub fn span_with(&self, tag: &Tag) -> &str {
        if self.after() <= tag.before() {
            &self.source[self.after()..tag.before()]
        } else {
            &self.source[tag.after()..self.before()]
        }
    }

    pub fn root(source: &'src str) -> Self {
        Tag {
            name: "",
            source,
            span: 0..0,
            before_text: "",
            kind: TType::Opening,
        }
    }

    pub fn iter(&self) -> Res<TagIter<'src>> {
        Ok(TagIter::new(self))
    }

    pub fn before(&self) -> usize {
        self.span.start
    }

    pub fn after(&self) -> usize {
        self.span.end
    }

    pub fn repr(&self) -> &'src str {
        &self.source[self.span.start..self.span.end]
    }
}

#[test]
fn test_find_first() -> Res<()> {
    let hoge = Tag::get_first("<hoge>after hoge", "hoge")?.unwrap();
    assert_eq!(hoge.name, "hoge");
    assert_eq!(hoge.span, 0..6);
    let hoge = Tag::get_first("before hoge<hoge>after hoge", "hoge")?.unwrap();
    assert_eq!(hoge.span, 11..17);
    assert_eq!(hoge.before_text, "before hoge");
    assert_eq!(
        Tag::get_first(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uuid_id">
            <metadata
            xmlns:opf="http://www.idpf.org/2007/opf"
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:dcterms="http://purl.org/dc/terms/"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
            xmlns:calibre="http://calibre.kovidgoyal.net/2009/metadata"
            >
            <dc:title>やっほう</dc:title></metadata></package></?xml>"#,
            "manifest"
        )
        .unwrap(),
        None
    );
    Ok(())
}

#[test]
fn test_get_end_1() -> Res<()> {
    let source = "aa<span>bb</span>cc";

    let mut iter = Tag::root(source).iter()?;
    while let Some(tag) = iter.next_by_el(&[])? {
        let (_, _) = tag.get_end()?;
    }

    Ok(())
}

#[test]
fn test_get_end_2() -> Res<()> {
    let source = "aa<span>bb</span>cc<span>dd</span>ee";

    let span_1 = Tag::get_first(source, "span")?.unwrap();
    assert_eq!(span_1.get_end()?.1, "bb");

    Ok(())
}

#[test]
fn test_get_end_3() -> Res<()> {
    let source = "aa<hr/>bb";

    let hr = Tag::get_first(source, "hr")?.unwrap();
    assert_eq!(hr.get_end()?.1, "");

    Ok(())
}

#[test]
fn test_find_incremental() -> Res<()> {
    let source = "<body><div><p>a</p><p>b</p></div></body>";
    let mut div = Tag::get_first(source, "div")?.unwrap().iter()?;
    assert_eq!(div.next_by_tag(&[])?.unwrap().name, "p"); // first p with a
    assert_eq!(div.next_by_tag(&[])?.unwrap().name, "p"); // closig
    assert_eq!(div.next_by_tag(&[])?.unwrap().name, "p"); // second p with b
    assert_eq!(div.next_by_tag(&[])?.unwrap().name, "p"); // closing
    assert_eq!(div.next_by_tag(&[])?.unwrap().name, "div"); // closing div
    assert_eq!(div.next_by_tag(&[])?, None);
    Ok(())
}
