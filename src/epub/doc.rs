use std::ops::Not;
use std::{collections::HashMap, error::Error, fmt::Display};

use crate::epub::PType;
use crate::epub::xhtml::Tag;
use crate::yomi::Yomi;

use super::Paragraph;
use super::Res;
use super::xhtml::TType;
use super::xhtml::iter::TagIter;

#[derive(Debug)]
enum DocError {
    Unschematic,
    UnknownFormating(String),
    DetailedError(String, usize, Box<dyn Error>),
}

impl Display for DocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocError::Unschematic => f.write_str("Invalid document: Unschematic!"),
            DocError::UnknownFormating(s) => write!(f, "Unknown formating: {s}!"),
            DocError::DetailedError(s, r, e) => write!(f, "Error: {s} {r} {e}!"),
        }
    }
}

impl Error for DocError {}

pub fn get_manifest(source: &str) -> Res<HashMap<&str, &str>> {
    let mut id_map = HashMap::new();
    let mut manifest = Tag::get_first(source, "manifest")?
        .ok_or(DocError::Unschematic)?
        .iter()?;
    while let Some(item) = manifest.next_by_tag(&["item"])? {
        let id = item.get_attr("id")?.ok_or(DocError::Unschematic)?;
        let href = item.get_attr("href")?.ok_or(DocError::Unschematic)?;
        id_map.insert(id, href);
    }
    Ok(id_map)
}

pub fn get_spine(source: &str) -> Res<Vec<&str>> {
    let mut idrefs = Vec::new();
    let mut spine = Tag::get_first(source, "spine")?
        .ok_or(DocError::Unschematic)?
        .iter()?;
    while let Some(item) = spine.next_by_tag(&["itemref"])? {
        let idref = item.get_attr("idref")?.ok_or(DocError::Unschematic)?;
        idrefs.push(idref);
    }

    Ok(idrefs)
}

pub fn get_toc(source: &str) -> Res<Vec<(String, String)>> {
    let mut navmap = Tag::get_first(source, "navMap")?
        .ok_or(DocError::Unschematic)?
        .iter()?;
    let mut chapters = Vec::new();

    while let Some(navpoint) = navmap.next_by_el(&["navPoint"])? {
        let label = navpoint
            .get_first_child("navLabel")?
            .ok_or(DocError::Unschematic)?;
        let text = label
            .get_first_child("text")?
            .ok_or(DocError::Unschematic)?;
        let (_, title) = text.get_end()?;
        let content = navpoint
            .get_first_child("content")?
            .ok_or(DocError::Unschematic)?;

        let src = content.get_attr("src")?.ok_or(DocError::Unschematic)?;
        let src_file = src.split_once('#').map(|(file, _)| file).unwrap_or(src);

        chapters.push((title.to_owned(), src_file.to_owned()));
    }

    Ok(chapters)
}

fn parse_paragraph<'src>(tag: &Tag<'src>) -> Res<Paragraph<'src>> {
    let (end_tag, inner) = tag.get_end()?;
    let inner = inner.trim();

    if ["div", "section"].contains(&tag.name) {
        return Ok(Paragraph {
            text: "",
            kind: PType::Transparent,
        });
    }
    if ["h1", "h2", "h3", "h4"].contains(&tag.name) {
        return Ok(Paragraph {
            text: inner,
            kind: PType::Header,
        });
    }

    if ["svg", "img"].contains(&tag.name) {
        return Ok(Paragraph {
            text: inner,
            kind: PType::StandaloneImage,
        });
    }

    if ["hr"].contains(&tag.name) {
        return Ok(Paragraph {
            text: inner,
            kind: PType::Empty,
        });
    }

    if ["p", "a", "span"].contains(&tag.name).not() {
        Err(DocError::UnknownFormating(tag.repr().to_owned()))?;
    }

    if let Some(img) = tag.iter()?.next_by_el(&["img", "svg"])? {
        if tag.span_with(&img).trim().is_empty() && img.span_with(&end_tag).trim().is_empty() {
            return Ok(Paragraph {
                text: inner,
                kind: PType::StandaloneImage,
            });
        }
    }

    if let Some(br) = tag.get_first_child("br")? {
        if tag.span_with(&br).trim().is_empty() && br.span_with(&end_tag).trim().is_empty() {
            return Ok(Paragraph {
                text: inner,
                kind: PType::Empty,
            });
        }
    }

    Ok(Paragraph {
        text: inner,
        kind: PType::BodyText,
    })
}

struct PassageParser<'src> {
    href: &'src str,
    body: Option<TagIter<'src>>,
}

impl<'src> Iterator for PassageParser<'src> {
    type Item = Res<Paragraph<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(body) = &mut self.body else {
            return Some(Err(DocError::Unschematic.into()));
        };
        while let Ok(Some(p)) = body.next_by_el(&[]) {
            let parsed = parse_paragraph(&p)
                .map_err(|e| DocError::DetailedError(self.href.to_owned(), p.before(), e).into());
            if let Ok(Paragraph {
                kind: PType::Transparent,
                ..
            }) = parsed
            {
                continue;
            }
            if p.kind == TType::Opening {
                let res = body.step_out(&p);
                return Some(res.and(parsed));
            }
            return Some(parsed);
        }
        None
    }
}

pub fn parse_passage<'src>(
    href: &'src str,
    source: &'src str,
) -> impl Iterator<Item = Res<Paragraph<'src>>> {
    let iter = Tag::get_first(source, "body")
        .and_then(|body| body.ok_or(DocError::Unschematic.into()))
        .and_then(|body| body.iter())
        .ok();
    PassageParser { href, body: iter }
}

pub fn with_fmt_stripped<'b, 'src>(
    gaiji: &mut HashMap<String, char>,
    yomi: &mut Vec<Yomi<'src>>,
    out: &'b mut String,
    p: &'src str,
) -> Res<&'b str> {
    let root = Tag::root(p);
    let mut iter = root.iter()?;
    while let Some(tag) = iter.next_by_tag(&[])? {
        out.push_str(tag.before_text);
        match tag.kind {
            TType::Closing => continue,
            TType::SelfClosing => match tag.name {
                "br" => out.push('\n'),
                "img" => {
                    let Some(src) = tag.get_attr("src")? else {
                        Err(DocError::UnknownFormating(tag.repr().to_owned()))?
                    };
                    let gaiji_ch = match gaiji.get(src) {
                        Some(&gaiji_ch) => gaiji_ch,
                        None => {
                            let replacement_ch = '�';
                            if let Some("gaiji" | "gaiji-line") = tag.get_attr("class")? {
                                gaiji.insert(src.to_owned(), replacement_ch);
                            } else {
                                Err(DocError::UnknownFormating(tag.repr().to_owned()))?;
                            }
                            replacement_ch
                        }
                    };
                    out.push(gaiji_ch);
                }
                _ => {
                    Err(DocError::UnknownFormating(tag.repr().to_owned()))?;
                }
            },
            TType::Opening => {
                let (end_tag, inner) = iter
                    .step_out(&tag)?
                    .ok_or_else(|| DocError::UnknownFormating(tag.repr().to_owned()))?;
                match tag.name {
                    "span" | "a" | "em" => out.push_str(inner),
                    "ruby" => {
                        let mut iter = tag.iter()?;
                        // In case there are rb tags, use the contents of them
                        let mut last_rb = None;
                        // In case there are no rb tags, and the base text starts from the end of ruby
                        let mut last_rt = out.len();
                        while let Some(r) = iter.next_by_el(&[])? {
                            out.push_str(r.before_text);
                            if r.kind == TType::Closing {
                                continue;
                            }
                            let (_, inner_r) = iter
                                .step_out(&r)?
                                .ok_or_else(|| DocError::UnknownFormating(tag.repr().to_owned()))?;
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
                                    Err(DocError::UnknownFormating(tag.repr().to_owned()))?;
                                }
                            }
                        }
                        out.push_str(end_tag.before_text);
                    }
                    _ => {
                        Err(DocError::UnknownFormating(tag.repr().to_owned()))?;
                    }
                }
            }
        }
    }
    out.push('\n');
    Ok(out.as_str())
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
    )
    .unwrap();
    assert_eq!(result, "金髪に染めた20歳くらいの男!!（だとか）\n");

    buf.clear();
    let result = with_fmt_stripped(
        &mut gaiji,
        &mut yomi,
        &mut buf,
        r#"<ruby><rb>山</rb><rt>やま</rt><rb>野</rb><rt>の</rt><rb>光</rb><rt>みつ</rt><rb>波</rb><rt>は</rt></ruby>、<span class="tcy">18</span>歳。"#,
    )
    .unwrap();
    assert_eq!(result, "山野光波、18歳。\n");

    buf.clear();
    let result = with_fmt_stripped(
        &mut gaiji,
        &mut yomi,
        &mut buf,
        r#"<ruby><rb>漢</rb><rb>字</rb><rt>kan</rt><rt>ji</rt></ruby>"#,
    )
    .unwrap();
    assert_eq!(result, "漢字\n");

    buf.clear();
    let result = with_fmt_stripped(
        &mut gaiji,
        &mut yomi,
        &mut buf,
        r#"<ruby>漢<rt>Kan</rt>字<rt>ji</rt>!</ruby>"#,
    )
    .unwrap();
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
    let mut body = Tag::get_first(source, "body")
        .unwrap()
        .unwrap()
        .iter()
        .unwrap();
    let p = body.next_by_el(&[]).unwrap().unwrap();
    assert_eq!(parse_paragraph(&p).unwrap().kind, PType::Transparent);
    let p = body.next_by_el(&[]).unwrap().unwrap();
    assert_eq!(parse_paragraph(&p).unwrap().kind, PType::StandaloneImage);
}

#[test]
fn test_parse_paragraph_2() {
    let source = r#"<body><p class="calibre3">　次から次へと、とんでもない言葉が口から衝いて出るジャティスに、フェロードも、グレンも、<ruby><rb>最</rb><rt>も</rt><rb>早</rb><rt>はや</rt></ruby>、脳内処理が追いつかない。</p>
    <p class="calibre3"><img class="fit" src="../images/00009.jpeg" alt=""/></p>
    <p class="calibre3">「ご、五億年……？」</p></body>"#;
    let mut body = Tag::get_first(source, "body")
        .unwrap()
        .unwrap()
        .iter()
        .unwrap();
    let p = body.next_by_el(&[]).unwrap().unwrap();
    assert_eq!(parse_paragraph(&p).unwrap().kind, PType::BodyText);
    body.step_out(&p).unwrap();
    let p = body.next_by_el(&[]).unwrap().unwrap();
    assert_eq!(parse_paragraph(&p).unwrap().kind, PType::StandaloneImage);
    body.step_out(&p).unwrap();
    let p = body.next_by_el(&[]).unwrap().unwrap();
    assert_eq!(parse_paragraph(&p).unwrap().kind, PType::BodyText);
    body.step_out(&p).unwrap();
}
