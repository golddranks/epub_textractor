use std::collections::HashMap;
use std::fs::File;

use crate::chapters::Chapter;
use crate::error::OrDie;
use crate::yomi::Yomi;
use crate::{即死, 死, PHASE};

mod doc;
mod xhtml;
mod zip;

pub struct Epub {
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub body: Vec<(String, String)>,
    pub href_to_spine_idx: HashMap<String, usize>,
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

        // hrefs is a manifest href -> spine idx map
        let mut href_to_spine_idx = HashMap::new();

        // texts is essentially the spine, but instead of ids, it has hrefs and xhtml file contents
        let mut body = Vec::new();

        for (idx, idref) in spine.iter().enumerate() {
            let href = manifest.get(idref).or_(死!("idref not found in manifest!"));
            let text_file = files.get(href).or_(死!("href not found in zipped files!"));
            let text_string = text_file.extract_string(file);
            href_to_spine_idx.insert(href.to_owned(), idx);
            body.push((href.to_owned(), text_string));
        }

        Epub {
            title,
            author,
            publisher,
            body,
            href_to_spine_idx,
            toc,
        }
    }

    pub fn paragraph_iter(&self, chapter: &Chapter) -> impl Iterator<Item = Paragraph> {
        self.body[chapter.idxs.clone()]
            .iter()
            .flat_map(|(href, passage)| doc::parse_passage(href, passage))
    }
}
