use std::collections::HashMap;

use crate::{
    Ctx,
    epub::{Epub, PType, Paragraph},
    yomi::Yomi,
};

pub fn produce_txt_yomi<'src>(
    ctx: &mut Ctx,
    gaiji: &mut HashMap<String, char>,
    epub: &'src Epub,
) -> (String, Vec<Yomi<'src>>) {
    ctx.phase = "produce";
    let mut yomi = Vec::new();
    let mut output = String::new();
    for (_, paragraphs) in epub.chapter_iter() {
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
                    p.with_fmt_stripped(ctx, gaiji, &mut yomi, &mut output);
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
