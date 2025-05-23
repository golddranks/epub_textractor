use std::collections::HashMap;

use crate::{
    PHASE,
    chapters::Chapter,
    epub::{Epub, PType, Paragraph},
    yomi::Yomi,
};

pub fn produce_txt_yomi<'src>(
    gaiji: &mut HashMap<String, char>,
    epub: &'src Epub,
    chapters: &[Chapter],
) -> (String, Vec<Yomi<'src>>) {
    let mut yomi = Vec::new();
    let mut output = String::new();
    for chapter in chapters.iter().filter(|c| !c.skip) {
        let paragraphs = epub.paragraph_iter(chapter);
        PHASE.set(format!("produce: {}", chapter.chap_name));
        let mut chapter_break_done = false;
        for paragraph in paragraphs {
            match paragraph {
                p @ Paragraph {
                    kind: PType::BodyText | PType::Header,
                    ..
                } => {
                    if !chapter_break_done {
                        output.push_str("\n\n\n\n");
                        chapter_break_done = true;
                    }
                    p.with_fmt_stripped(gaiji, &mut yomi, &mut output);
                }
                Paragraph {
                    kind: PType::Empty, ..
                } => output.push('\n'),
                Paragraph {
                    kind: PType::StandaloneImage | PType::Transparent,
                    ..
                } => (),
            }
        }
    }

    (output, yomi)
}
