use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::ops::{Not, Range};

use crate::chapters::{self, Chapter};
use crate::error::ResultOrDie;
use crate::yomi::Yomi;
use crate::{PHASE, 死};

mod doc;
mod xhtml;
mod zip;

type Res<T> = Result<T, Box<dyn Error>>;

pub struct Epub {
    pub chapters: Vec<Chapter>,
    pub texts: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PType {
    BodyText,
    Header,
    StandaloneImage,
    Empty,
    Transparent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paragraph<'src> {
    pub text: &'src str,
    pub kind: PType,
}

impl<'src> Paragraph<'src> {
    pub fn with_fmt_stripped<'b>(
        &self,
        gaiji: &mut HashMap<String, char>,
        yomi: &mut Vec<Yomi<'src>>,
        buf: &'b mut String,
    ) -> &'b str {
        doc::with_fmt_stripped(gaiji, yomi, buf, self.text).or_die(|e| 死!(e))
    }
}

pub fn extract_contents(file: &mut File, mut chapters: Vec<Chapter>) -> Epub {
    PHASE.set("extract_contents");
    let mut files = HashMap::new();
    let mut toc = None;
    let mut content = None;
    for file in zip::FileIter::new(file) {
        let file = file.or_die(|e| 死!(e));
        match &*file.name {
            "content.opf" => content = Some(file),
            "toc.ncx" => toc = Some(file),
            name => {
                files.insert(name.to_owned(), file);
            }
        }
    }

    let (Some(toc), Some(content)) = (toc, content) else {
        死!("No toc.ncx or content.opf found!");
    };

    let toc = toc.extract_string(file).or_die(|e| 死!(e));
    let content = content.extract_string(file).or_die(|e| 死!(e));

    let manifest = doc::get_manifest(&content).or_die(|e| 死!(e));
    let spine = doc::get_spine(&content).or_die(|e| 死!(e));

    let mut texts = Vec::new();
    let mut hrefs = HashMap::new();

    for (idx, idref) in spine.iter().enumerate() {
        let href = manifest[idref];
        let text_file = &files[href];
        let text_string = text_file.extract_string(file).or_die(|e| 死!(e));
        hrefs.insert(href, idx);
        texts.push((href.to_owned(), text_string));
    }

    let toc = doc::get_toc(&toc).or_die(|e| 死!(e));

    chapters::update_chapters(&mut chapters, &toc, &hrefs, &texts);

    Epub { texts, chapters }
}

impl Epub {
    pub fn chapter_iter(&self) -> impl Iterator<Item = (&str, impl Iterator<Item = Paragraph>)> {
        self.chapters
            .iter()
            .filter(|chapter| chapter.skip.not())
            .map(|chapter| {
                let title = chapter.name.as_str();
                let range = chapter.idxs.clone();
                if range.start > range.end {
                    eprintln!(
                        "Chapter {title} has weird ranges, {}..{}",
                        range.start, range.end
                    );
                }
                (title, self.paragraph_iter(range))
            })
    }

    pub fn paragraph_iter(&self, range: Range<usize>) -> impl Iterator<Item = Paragraph> {
        self.texts[range]
            .iter()
            .flat_map(|(href, passage)| doc::parse_passage(href, passage))
            .map(|r| r.or_die(|e| 死!(e)))
    }
}
