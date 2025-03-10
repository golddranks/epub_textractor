use std::{collections::HashMap, io::Write, ops::Range, path::Path};

use crate::{
    error::{OptionOrDie, ResultOrDie},
    死,
};

#[derive(Debug)]
pub struct Chapter {
    pub name: String,
    pub idxs: Range<usize>,
    pub files: Vec<String>,
    pub skip: bool,
}

pub fn read_chapters(chapters_fname: &Path) -> Option<Vec<Chapter>> {
    let Ok(file) = std::fs::read_to_string(chapters_fname) else {
        return None;
    };
    let mut chapters = Vec::new();
    for line in file.lines() {
        let mut fields = line.split(':');
        let name = fields
            .next()
            .or_die(|| 死!("Invalid chapters file"))
            .to_owned();
        let skip = fields.next().or_die(|| 死!("Invalid chapters file"));
        let idx_start: usize = fields
            .next()
            .or_die(|| 死!("Invalid chapters file"))
            .parse()
            .or_die(|e| 死!("Invalid chapters file: {e}"));
        let idx_end: usize = fields
            .next()
            .or_die(|| 死!("Invalid chapters file"))
            .parse()
            .or_die(|e| 死!("Invalid chapters file: {e}"));
        let files = fields.map(ToOwned::to_owned).collect();

        chapters.push(Chapter {
            name,
            idxs: idx_start..idx_end,
            files,
            skip: match skip {
                "SKIP" => true,
                "TAKE" => false,
                _ => 死!("Invalid chapters file"),
            },
        });
    }
    Some(chapters)
}

pub fn write_chapters(chapters: &[Chapter], mut file: impl Write) {
    for chapter in chapters {
        write!(file, "{}", chapter.name).or_die(|e| 死!(e));
        if chapter.skip {
            write!(file, ":SKIP").or_die(|e| 死!(e));
        } else {
            write!(file, ":TAKE").or_die(|e| 死!(e));
        }
        write!(file, ":{}:{}", chapter.idxs.start, chapter.idxs.end).or_die(|e| 死!(e));
        for fname in &chapter.files {
            write!(file, ":{}", fname).or_die(|e| 死!(e));
        }
        writeln!(file).or_die(|e| 死!(e));
    }
}

fn generate(
    chapters: &mut Vec<Chapter>,
    name: &str,
    idxs: Range<usize>,
    texts: &[(String, String)],
    atogaki_seen: &mut bool,
) {
    let mut skip = false;
    let mut files = Vec::new();
    if idxs.end < idxs.start {
        skip = true;
    } else {
        files = texts[idxs.clone()]
            .iter()
            .map(|(fname, _)| fname.to_owned())
            .collect();
    }
    match &*name {
        "表紙" | "目次" | "ＣＯＮＴＥＮＴＳ" | "contents" | "Contents" | "CONTENTS" => {
            skip = true;
        }
        "あとがき" | "後書き" | "後書" => {
            *atogaki_seen = true;
            skip = true;
        }
        _ if *atogaki_seen => skip = true,
        _ => (),
    }
    chapters.push(Chapter {
        name: name.to_owned(),
        idxs,
        files,
        skip,
    })
}

pub fn update_chapters(
    chapters: &mut Vec<Chapter>,
    toc: &[(String, String)],
    hrefs: &HashMap<&str, usize>,
    texts: &[(String, String)],
) {
    let mut atogaki_seen = false;
    if chapters.len() == 0 {
        for chapter in toc.windows(2) {
            let [current, next] = chapter else {
                unreachable!()
            };
            let start_idx = hrefs.get(&*current.1).unwrap_or(&0);
            let end_idx = hrefs.get(&*next.1).unwrap_or(&0);
            let idxs = *start_idx..*end_idx;
            generate(chapters, &current.0, idxs, texts, &mut atogaki_seen);
        }
        let Some(last) = toc.last() else {
            死!("no chapters?")
        };
        let start_idx = hrefs.get(&*last.1).or_die(|| 死!("no href found?"));
        generate(
            chapters,
            &last.0,
            *start_idx..texts.len(),
            texts,
            &mut atogaki_seen,
        );
    }
}
