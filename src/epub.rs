use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::ops::Range;

use crate::Res;
use crate::yomi::Yomi;

mod doc;
mod xhtml;
mod zip;

#[derive(Debug, Clone, Copy)]
enum EpubError {
    NoTocOrContentFound,
}

impl Display for EpubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpubError::NoTocOrContentFound => f.write_str("No toc.ncx or content.opf found!"),
        }
    }
}

impl Error for EpubError {}

pub struct Epub {
    pub chapters: Vec<(String, usize)>,
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
    ) -> Res<&'b str> {
        doc::with_fmt_stripped(gaiji, yomi, buf, self.text)
    }
}

pub fn extract_contents(file: &mut File) -> Res<Epub> {
    let mut files = HashMap::new();
    let mut toc = None;
    let mut content = None;
    for file in zip::FileIter::new(file) {
        let file = file?;
        match &*file.name {
            "content.opf" => content = Some(file),
            "toc.ncx" => toc = Some(file),
            name => {
                files.insert(name.to_owned(), file);
            }
        }
    }

    let (Some(toc), Some(content)) = (toc, content) else {
        return Err(EpubError::NoTocOrContentFound)?;
    };

    let toc = toc.extract_string(file)?;
    let content = content.extract_string(file)?;

    let manifest = doc::get_manifest(&content)?;
    let spine = doc::get_spine(&content)?;

    let mut texts = Vec::new();
    let mut hrefs = HashMap::new();

    for (idx, idref) in spine.iter().enumerate() {
        let href = manifest[idref];
        let text_file = &files[href];
        let text_string = text_file.extract_string(file)?;
        hrefs.insert(href, idx);
        texts.push((href.to_owned(), text_string));
    }

    let chapters = doc::get_chapters(&toc, &hrefs)?;

    Ok(Epub { chapters, texts })
}

impl Epub {
    pub fn chapter_iter(
        &self,
    ) -> impl Iterator<Item = (&str, impl Iterator<Item = Res<Paragraph>>)> {
        self.chapters.windows(2).map(|range| {
            (
                range[0].0.as_str(),
                self.paragraph_iter(range[0].1..range[1].1),
            )
        })
    }

    pub fn paragraph_iter(&self, range: Range<usize>) -> impl Iterator<Item = Res<Paragraph>> {
        self.texts[range]
            .iter()
            .flat_map(|(href, passage)| doc::parse_passage(href, passage))
    }
}
