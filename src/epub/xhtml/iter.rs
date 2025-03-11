use std::ops::Not;

use crate::{error::OrDie, 即死, 死};

use super::{TType, Tag, tag_parser::parse_tag};

#[derive(Debug)]
pub struct TagIter<'src> {
    stack: Vec<(usize, &'src str)>,
    pos: usize,
    root: Tag<'src>,
}

impl<'src> TagIter<'src> {
    pub fn new(tag: &Tag<'src>) -> Self {
        TagIter {
            stack: vec![(tag.after(), tag.name)],
            pos: tag.span.end,
            root: tag.clone(),
        }
    }

    pub fn next_by_el(&mut self, target_els: &[&str]) -> Option<Tag<'src>> {
        while let Some(tag) = self.next_by_tag(target_els) {
            if tag.kind != TType::Closing {
                return Some(tag);
            }
        }
        None
    }

    pub fn next_by_tag(&mut self, target_tags: &[&str]) -> Option<Tag<'src>> {
        if self.root.kind != TType::Opening {
            return None;
        }
        while self.stack.is_empty().not() {
            let tag = match parse_tag(self.root.source, self.pos) {
                Some(tag) => tag,
                None => {
                    // no literal tags left in source, trying for root tag
                    if let Some((_, "")) = self.stack.last() {
                        Tag {
                            name: "",
                            source: self.root.source,
                            span: self.root.source.len()..self.root.source.len(),
                            before_text: &self.root.source[self.pos..],
                            kind: TType::Closing,
                        }
                    } else {
                        // source endeded, but there were still unpopped tags in stack?
                        即死!("unexpected EOF");
                    }
                }
            };
            self.pos = tag.after();
            if tag.kind == TType::Opening {
                self.stack.push((self.pos, tag.name));
            }
            if tag.kind == TType::Closing {
                match self.stack.pop() {
                    // closing tag in source matches the tag on stack
                    Some((_, expected)) if expected == tag.name => (),
                    _ => 即死!("closing tag mismatch"),
                }
            };
            if target_tags.is_empty() || target_tags.contains(&tag.name) {
                return Some(tag);
            }
        }
        None
    }

    pub fn step_out(&mut self, tag: &Tag<'src>) -> Option<(Tag<'src>, &'src str)> {
        if tag.kind == TType::SelfClosing && self.pos == tag.after() {
            return None;
        }
        let Some(tag_depth) = self
            .stack
            .iter()
            .position(|&(pos, name)| pos == tag.after() && name == tag.name)
        else {
            即死!("unexpected not found");
        };

        let end_tag = loop {
            let end_tag = self.next_by_tag(&[tag.name]).or_(死!("unexpected EOF"));
            if self.stack.len() == tag_depth {
                break end_tag;
            }
        };

        let inner = &self.root.source[tag.after()..end_tag.before()];
        Some((end_tag, inner))
    }
}

#[test]
fn test_tag_iter_step_out() {
    let source = r#"染めた<span>20</span>歳くらいの男<span>!!</span>（だとか）"#;

    let mut iter = Tag::root(source).iter();
    let tag = iter.next_by_el(&[]).unwrap();
    let (_, inner) = iter.step_out(&tag).unwrap();
    assert_eq!(inner, "20");
    let tag = iter.next_by_el(&[]).unwrap();
    let (_, inner) = iter.step_out(&tag).unwrap();
    assert_eq!(inner, "!!");
    assert_eq!(iter.next_by_el(&[]), None);
}

#[test]
fn test_tag_iter_just_next() {
    let source = r#"染めた<span>20</span>歳くらいの男<span>!!</span>（だとか）"#;

    let mut iter = Tag::root(source).iter();
    assert_eq!(iter.next_by_el(&[]).unwrap().name, "span");
    assert_eq!(iter.next_by_el(&[]).unwrap().name, "span");
    assert_eq!(iter.next_by_el(&[]), None);
}
