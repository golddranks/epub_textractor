use std::collections::HashMap;
use std::ops::Not;

use crate::PHASE;
use crate::epub::PType;
use crate::epub::xhtml::Tag;
use crate::error::{OrDie, 即死, 死};
use crate::yomi::Yomi;

use super::Paragraph;
use super::xhtml::TType;
use super::xhtml::iter::TagIter;

pub fn get_manifest(source: &str) -> HashMap<String, String> {
    let mut id_map = HashMap::new();
    let mut manifest = Tag::get_first(source, "manifest")
        .or_(死!("unschematic"))
        .iter();
    while let Some(item) = manifest.next_by_tag(&["item"]) {
        let id = item.get_attr("id").or_(死!("unschematic"));
        let href = item.get_attr("href").or_(死!("unschematic"));
        id_map.insert(id.to_owned(), href.to_owned());
    }
    id_map
}

pub fn get_spine(source: &str) -> Vec<String> {
    let mut idrefs = Vec::new();
    let mut spine = Tag::get_first(source, "spine")
        .or_(死!("unschematic"))
        .iter();
    while let Some(item) = spine.next_by_tag(&["itemref"]) {
        let idref = item.get_attr("idref").or_(死!("unschematic"));
        idrefs.push(idref.to_owned());
    }

    idrefs
}

pub fn get_toc(source: &str) -> Vec<(String, String)> {
    let mut navmap = Tag::get_first(source, "navMap")
        .or_(死!("unschematic"))
        .iter();
    let mut chapters = Vec::new();

    while let Some(navpoint) = navmap.next_by_el(&["navPoint"]) {
        let label = navpoint.get_first_child("navLabel").or_(死!("unschematic"));
        let text = label.get_first_child("text").or_(死!("unschematic"));
        let (_, title) = text.get_end();
        let content = navpoint.get_first_child("content").or_(死!("unschematic"));

        let src = content.get_attr("src").or_(死!("unschematic"));
        let src_file = src.split_once('#').map(|(file, _)| file).unwrap_or(src);

        chapters.push((title.to_owned(), src_file.to_owned()));
    }

    chapters
}

fn parse_paragraph<'src>(tag: &Tag<'src>) -> Paragraph<'src> {
    let (end_tag, inner) = tag.get_end();
    let inner = inner.trim();

    if ["div", "section"].contains(&tag.name) {
        return Paragraph {
            text: "",
            kind: PType::Transparent,
        };
    }
    if ["h1", "h2", "h3", "h4"].contains(&tag.name) {
        return Paragraph {
            text: inner,
            kind: PType::Header,
        };
    }

    if ["svg", "img"].contains(&tag.name) {
        return Paragraph {
            text: inner,
            kind: PType::StandaloneImage,
        };
    }

    if ["hr"].contains(&tag.name) {
        return Paragraph {
            text: inner,
            kind: PType::Empty,
        };
    }

    if ["p", "a", "span"].contains(&tag.name).not() {
        即死!("unknown formatting");
    }

    if let Some(img) = tag.iter().next_by_el(&["img", "svg"]) {
        if tag.span_with(&img).trim().is_empty() && img.span_with(&end_tag).trim().is_empty() {
            return Paragraph {
                text: inner,
                kind: PType::StandaloneImage,
            };
        }
    }

    if let Some(br) = tag.get_first_child("br") {
        if tag.span_with(&br).trim().is_empty() && br.span_with(&end_tag).trim().is_empty() {
            return Paragraph {
                text: inner,
                kind: PType::Empty,
            };
        }
    }

    Paragraph {
        text: inner,
        kind: PType::BodyText,
    }
}

struct PassageParser<'src> {
    body: TagIter<'src>,
}

impl<'src> Iterator for PassageParser<'src> {
    type Item = Paragraph<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(p) = self.body.next_by_el(&[]) {
            let parsed = parse_paragraph(&p);
            if let Paragraph {
                kind: PType::Transparent,
                ..
            } = parsed
            {
                continue;
            }
            if p.kind == TType::Opening {
                self.body.step_out(&p);
                return Some(parsed);
            }
            return Some(parsed);
        }
        None
    }
}

pub fn parse_passage<'src>(
    href: &'src str,
    source: &'src str,
) -> impl Iterator<Item = Paragraph<'src>> {
    PHASE.set(format!("produce: {href}"));
    let iter = Tag::get_first(source, "body")
        .or_(死!("unschematic"))
        .iter();
    PassageParser { body: iter }
}

pub fn with_fmt_stripped<'b, 'src>(
    gaiji: &mut HashMap<String, char>,
    yomi: &mut Vec<Yomi<'src>>,
    out: &'b mut String,
    p: &'src str,
) -> &'b str {
    let root = Tag::root(p);
    let mut iter = root.iter();
    while let Some(tag) = iter.next_by_tag(&[]) {
        out.push_str(tag.before_text);
        match tag.kind {
            TType::Closing => continue,
            TType::SelfClosing => match tag.name {
                "br" => out.push('\n'),
                "img" => {
                    let src = tag.get_attr("src").or_(死!("unknown formatting"));
                    let gaiji_ch = match gaiji.get(src) {
                        Some(&gaiji_ch) => gaiji_ch,
                        None => {
                            let replacement_ch = '�';
                            if let Some("gaiji" | "gaiji-line") = tag.get_attr("class") {
                                gaiji.insert(src.to_owned(), replacement_ch);
                            } else {
                                即死!("unknown formatting");
                            }
                            replacement_ch
                        }
                    };
                    out.push(gaiji_ch);
                }
                _ => {
                    即死!("unknown formatting");
                }
            },
            TType::Opening => {
                let (end_tag, inner) = iter.step_out(&tag).or_(死!("unknown formatting"));
                match tag.name {
                    "span" | "a" | "em" => out.push_str(inner),
                    "ruby" => {
                        let mut iter = tag.iter();
                        // In case there are rb tags, use the contents of them
                        let mut last_rb = None;
                        // In case there are no rb tags, and the base text starts from the end of ruby
                        let mut last_rt = out.len();
                        while let Some(r) = iter.next_by_el(&[]) {
                            out.push_str(r.before_text);
                            if r.kind == TType::Closing {
                                continue;
                            }
                            let (_, inner_r) = iter.step_out(&r).or_(死!("unknown formatting"));
                            match r.name {
                                "rb" => {
                                    last_rb = Some(out.len()..out.len() + inner_r.len());
                                    out.push_str(inner_r);
                                }
                                "rt" => {
                                    let rb_span = last_rb.unwrap_or(last_rt..out.len());
                                    yomi.push(Yomi {
                                        span: rb_span,
                                        rt: inner_r,
                                    });
                                    last_rt = out.len();
                                    last_rb = None;
                                }
                                _ => {
                                    即死!("unknown formatting");
                                }
                            }
                        }
                        out.push_str(end_tag.before_text);
                    }
                    _ => {
                        即死!("unknown formatting");
                    }
                }
            }
        }
    }
    out.push('\n');
    out.as_str()
}

#[test]
fn test_strip_formating() {
    let mut buf = String::new();
    let mut gaiji = HashMap::new();
    let mut yomi = Vec::new();

    let result = with_fmt_stripped(
        &mut gaiji,
        &mut yomi,
        &mut buf,
        r#"金髪に染めた<span class="tcy">20</span>歳くらいの男<span class="tcy">!!</span>（だとか）"#,
    );
    assert_eq!(result, "金髪に染めた20歳くらいの男!!（だとか）\n");

    buf.clear();
    let result = with_fmt_stripped(
        &mut gaiji,
        &mut yomi,
        &mut buf,
        r#"<ruby><rb>山</rb><rt>やま</rt><rb>野</rb><rt>の</rt><rb>光</rb><rt>みつ</rt><rb>波</rb><rt>は</rt></ruby>、<span class="tcy">18</span>歳。"#,
    );
    assert_eq!(result, "山野光波、18歳。\n");

    buf.clear();
    let result = with_fmt_stripped(
        &mut gaiji,
        &mut yomi,
        &mut buf,
        r#"<ruby><rb>漢</rb><rb>字</rb><rt>kan</rt><rt>ji</rt></ruby>"#,
    );
    assert_eq!(result, "漢字\n");

    buf.clear();
    let result = with_fmt_stripped(
        &mut gaiji,
        &mut yomi,
        &mut buf,
        r#"<ruby>漢<rt>Kan</rt>字<rt>ji</rt>!</ruby>"#,
    );
    assert_eq!(result, "漢字!\n");
}

#[test]
fn test_parse_paragraph_1() {
    let source = r##"<?xml version='1.0' encoding='utf-8'?>
    <html><body class="p-image">
      <div class="main">
        <svg>
          <image/>
        </svg>
      </div>
    </body></html>
"##;
    let mut body = Tag::get_first(source, "body").unwrap().iter();
    let p = body.next_by_el(&[]).unwrap();
    assert_eq!(parse_paragraph(&p).kind, PType::Transparent);
    let p = body.next_by_el(&[]).unwrap();
    assert_eq!(parse_paragraph(&p).kind, PType::StandaloneImage);
}

#[test]
fn test_parse_paragraph_2() {
    let source = r#"<body><p class="calibre3">　次から次へと、とんでもない言葉が口から衝いて出るジャティスに、フェロードも、グレンも、<ruby><rb>最</rb><rt>も</rt><rb>早</rb><rt>はや</rt></ruby>、脳内処理が追いつかない。</p>
    <p class="calibre3"><img class="fit" src="../images/00009.jpeg" alt=""/></p>
    <p class="calibre3">「ご、五億年……？」</p></body>"#;
    let mut body = Tag::get_first(source, "body").unwrap().iter();
    let p = body.next_by_el(&[]).unwrap();
    assert_eq!(parse_paragraph(&p).kind, PType::BodyText);
    body.step_out(&p).unwrap();
    let p = body.next_by_el(&[]).unwrap();
    assert_eq!(parse_paragraph(&p).kind, PType::StandaloneImage);
    body.step_out(&p).unwrap();
    let p = body.next_by_el(&[]).unwrap();
    assert_eq!(parse_paragraph(&p).kind, PType::BodyText);
    body.step_out(&p).unwrap();
}

pub fn get_author(source: &str) -> &str {
    let author = Tag::get_first(source, "dc:creator").or_(死!("unschematic"));
    author.get_end().1
}

pub fn get_title(source: &str) -> &str {
    let author = Tag::get_first(source, "dc:title").or_(死!("unschematic"));
    author.get_end().1
}
