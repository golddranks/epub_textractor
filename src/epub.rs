use std::collections::HashMap;
use std::fs::File;

use crate::chapters::Chapter;
use crate::yomi::Yomi;
use crate::{PHASE, 即死};

mod doc;
mod xhtml;
mod zip;

pub struct Epub {
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub texts: Vec<(String, String)>,
    pub hrefs: HashMap<String, usize>,
    pub spine: Vec<String>,
    pub toc: Vec<(String, String)>,
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
        doc::with_fmt_stripped(gaiji, yomi, buf, self.text)
    }
}

impl Epub {
    pub fn new(file: &mut File) -> Epub {
        PHASE.set("extract_contents");
        let mut files = HashMap::new();
        let mut toc = None;
        let mut content = None;
        for file in zip::FileIter::new(file) {
            match &*file.name {
                "content.opf" => content = Some(file),
                "toc.ncx" => toc = Some(file),
                name => {
                    files.insert(name.to_owned(), file);
                }
            }
        }

        let (Some(toc), Some(content)) = (toc, content) else {
            即死!("No toc.ncx or content.opf found!");
        };

        let toc = toc.extract_string(file);
        let content = content.extract_string(file);

        let title = doc::get_title(&content).to_owned();
        let author = doc::get_author(&content).to_owned();
        let publisher = doc::get_publisher(&content).to_owned();

        // manifest is a id->href map of the EPUB file contents (including images, style sheets, metadata etc.)
        let manifest = doc::get_manifest(&content);

        // spine is a list of ids that are in the reading order
        let spine = doc::get_spine(&content);

        // toc is a list of (chapter title, href) tuples, defining the starting point of each chapter
        let toc = doc::get_toc(&toc);

        // hrefs is a href->idx map of the spine
        let mut hrefs = HashMap::new();

        // texts is a list of tuples of href and corresponding text file contents in the spine order
        let mut texts = Vec::new();

        for (idx, idref) in spine.iter().enumerate() {
            let href = &manifest[idref];
            let text_file = &files[href];
            let text_string = text_file.extract_string(file);
            hrefs.insert(href.to_owned(), idx);
            texts.push((href.to_owned(), text_string));
        }

        Epub {
            title,
            author,
            publisher,
            texts,
            hrefs,
            spine,
            toc,
        }
    }

    pub fn paragraph_iter(&self, chapter: &Chapter) -> impl Iterator<Item = Paragraph> {
        self.texts[chapter.idxs.clone()]
            .iter()
            .flat_map(|(href, passage)| doc::parse_passage(href, passage))
    }
}
