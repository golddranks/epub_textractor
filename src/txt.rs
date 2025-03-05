use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{
    Res,
    epub::{Epub, PType, Paragraph},
    yomi::Yomi,
};

#[derive(Debug, Clone)]
struct MyError {
    msg: &'static str,
    details: String,
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyError {} {}", self.msg, self.details)
    }
}

impl Error for MyError {}

pub fn produce_txt_yomi<'src>(
    gaiji: &mut HashMap<String, char>,
    epub: &'src Epub,
) -> Res<(String, Vec<Yomi<'src>>)> {
    let mut yomi = Vec::new();
    let mut output = String::new();
    for (title, paragraphs) in epub.chapter_iter() {
        let mut chapter_break_done = false;
        for paragraph in paragraphs {
            match paragraph.map_err(|e| MyError {
                msg: "paragraph error",
                details: e.to_string() + " " + title,
            })? {
                p @ Paragraph {
                    kind: PType::BodyText | PType::Header,
                    ..
                } => {
                    if !chapter_break_done {
                        output.push_str("\n\n\n\n");
                        chapter_break_done = true;
                    }
                    p.with_fmt_stripped(gaiji, &mut yomi, &mut output)
                        .map_err(|e| MyError {
                            msg: "fmt stripping error",
                            details: e.to_string() + title,
                        })?;
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

    Ok((output, yomi))
}
