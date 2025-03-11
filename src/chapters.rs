use std::{io::Write, ops::Range, path::Path};

use crate::{
    epub::Epub, error::{OptionOrDie, ResultOrDie}, heuristics, 死
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Cover,
    Index,
    Prologue,
    Main ,
    Epilogue,
    BonusChapter,
    Afterword,
    Extra,
    Copyright,
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s {
            "cover" => Role::Cover,
            "index" => Role::Index,
            "prologue" => Role::Prologue,
            "main" => Role::Main,
            "epilogue" => Role::Epilogue,
            "bonus_chapter" => Role::BonusChapter,
            "afterword" => Role::Afterword,
            "extra" => Role::Extra,
            "copyright" => Role::Copyright,
            _ => 死!("Invalid role: {s}"),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Role::Cover => "cover",
            Role::Index => "index",
            Role::Prologue => "prologue",
            Role::Main => "main",
            Role::Epilogue => "epilogue",
            Role::BonusChapter => "bonus_chapter",
            Role::Afterword => "afterword",
            Role::Extra => "extra",
            Role::Copyright => "copyright",
        }
    }
}


#[derive(Debug)]
pub struct Chapter {
    pub name: String,
    pub idxs: Range<usize>,
    pub files: Vec<String>,
    pub role: Role,
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
            .or_die(|| 死!("Invalid name field in chapters file"))
            .to_owned();
        let role = fields.next().or_die(|| 死!("Invalid role field in chapters file"));
        let skip: &str = fields.next().or_die(|| 死!("Invalid skip field in chapters file"));
        let skip = match skip {
            "SKIP" => true,
            "TAKE" => false,
            _ => 死!("Invalid skip field in chapters file"),
        };
        let idx_start: usize = fields
            .next()
            .or_die(|| 死!("Invalid idx_start field in chapters file"))
            .parse()
            .or_die(|e| 死!("Invalid idx_start field in chapters file: {e}"));
        let idx_end: usize = fields
            .next()
            .or_die(|| 死!("Invalid idx_end field in chapters file"))
            .parse()
            .or_die(|e| 死!("Invalid idx_end field in chapters file: {e}"));
        let files = fields.map(ToOwned::to_owned).collect();

        chapters.push(Chapter {
            name,
            idxs: idx_start..idx_end,
            files,
            role: Role::from_str(role),
            skip
        });
    }
    Some(chapters)
}

pub fn write_chapters(chapters: &[Chapter], mut file: impl Write) {
    for chapter in chapters {
        write!(file, "{}", chapter.name).or_die(|e| 死!(e));
        write!(file, ":{}", chapter.role.to_str()).or_die(|e| 死!(e));
        write!(file, ":{}", match chapter.skip {
            true => "SKIP",
            false => "TAKE",
        }).or_die(|e| 死!(e));
        write!(file, ":{}:{}", chapter.idxs.start, chapter.idxs.end).or_die(|e| 死!(e));
        for fname in &chapter.files {
            write!(file, ":{}", fname).or_die(|e| 死!(e));
        }
        writeln!(file).or_die(|e| 死!(e));
    }
}

pub fn generate(
    epub: &Epub,
) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    let Epub { hrefs, toc, spine, .. } = epub;
    for chapter in toc.windows(2) {
        let [(name, href), (_, next_href)] = chapter else {
            unreachable!()
        };
        let start_idx = hrefs.get(href).unwrap_or(&0);
        let end_idx = hrefs.get(next_href).unwrap_or(&0);
        let idxs = *start_idx..*end_idx;
        let role = heuristics::guess_role(&chapters, name);
        chapters.push(Chapter{
            name: name.to_owned(),
            idxs: idxs.clone(),
            files: spine.get(idxs.clone()).or_die(|| 死!("weird order of files in TOC!")).to_owned(),
            role,
            skip: heuristics::is_skip(role)
        });
    }
    let Some((name, href)) = toc.last() else {
        死!("no chapters?")
    };
    let start_idx = hrefs.get(href).or_die(|| 死!("no href found?"));
    let idxs = *start_idx..spine.len();
    let role = heuristics::guess_role(&chapters, name);
    chapters.push(Chapter{
        name: name.to_owned(),
        idxs: idxs.clone(),
        files: spine[idxs].to_owned(),
        role,
        skip: heuristics::is_skip(role)
    });
    chapters
}
