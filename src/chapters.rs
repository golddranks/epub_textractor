use std::{fmt::Display, fs::File, io::Write, ops::Range, path::Path};

use crate::{epub::Epub, error::OrDie, heuristics, 即死, 死};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Cover,
    Index,
    Prologue,
    Main,
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
            _ => 即死!("Invalid role: {s}"),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Role::Cover => "cover",
            Role::Index => "index",
            Role::Prologue => "prologue",
            Role::Main => "main",
            Role::Epilogue => "epilogue",
            Role::BonusChapter => "bonus_chapter",
            Role::Afterword => "afterword",
            Role::Extra => "extra",
            Role::Copyright => "copyright",
        })
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

pub fn read(fname: &Path) -> Option<Vec<Chapter>> {
    let Ok(file) = std::fs::read_to_string(fname) else {
        return None;
    };
    let mut chapters = Vec::new();
    for line in file.lines() {
        let mut fields = line.split(':');
        let name = fields
            .next()
            .or_(死!("Invalid name field in chapters file"))
            .to_owned();
        let role = fields
            .next()
            .or_(死!("Invalid role field in chapters file"));
        let skip: &str = fields
            .next()
            .or_(死!("Invalid skip field in chapters file"));
        let skip = match skip {
            "SKIP" => true,
            "TAKE" => false,
            _ => 即死!("Invalid skip field in chapters file"),
        };
        let idx_start: usize = fields
            .next()
            .or_(死!("Invalid idx_start field in chapters file"))
            .parse()
            .or_(死!("Invalid idx_start field in chapters file"));
        let idx_end: usize = fields
            .next()
            .or_(死!("Invalid idx_end field in chapters file"))
            .parse()
            .or_(死!("Invalid idx_end field in chapters file"));
        let files = fields.map(ToOwned::to_owned).collect();

        chapters.push(Chapter {
            name,
            idxs: idx_start..idx_end,
            files,
            role: Role::from_str(role),
            skip,
        });
    }
    Some(chapters)
}

pub fn write(chapters: &[Chapter], fname: &Path) {
    let mut file = File::create(&fname).or_(死!());
    for chapter in chapters {
        write!(file, "{}", chapter.name).or_(死!());
        write!(file, ":{}", chapter.role).or_(死!());
        let skip = match chapter.skip {
            true => "SKIP",
            false => "TAKE",
        };
        write!(file, ":{}", skip).or_(死!());
        write!(file, ":{}:{}", chapter.idxs.start, chapter.idxs.end).or_(死!());
        for fname in &chapter.files {
            write!(file, ":{}", fname).or_(死!());
        }
        writeln!(file).or_(死!());
    }
}

pub fn generate(epub: &Epub) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    let Epub {
        hrefs, toc, spine, ..
    } = epub;
    for chapter in toc.windows(2) {
        let [(name, href), (_, next_href)] = chapter else {
            unreachable!()
        };
        let start_idx = hrefs.get(href).unwrap_or(&0);
        let end_idx = hrefs.get(next_href).unwrap_or(&0);
        let idxs = *start_idx..*end_idx;
        let role = heuristics::guess_role(&chapters, name);
        chapters.push(Chapter {
            name: name.to_owned(),
            idxs: idxs.clone(),
            files: spine
                .get(idxs.clone())
                .or_(死!("weird order of files in TOC!"))
                .to_owned(),
            role,
            skip: heuristics::is_skip(role),
        });
    }
    let Some((name, href)) = toc.last() else {
        即死!("no chapters?")
    };
    let start_idx = hrefs.get(href).or_(死!("no href found?"));
    let idxs = *start_idx..spine.len();
    let role = heuristics::guess_role(&chapters, name);
    chapters.push(Chapter {
        name: name.to_owned(),
        idxs: idxs.clone(),
        files: spine[idxs].to_owned(),
        role,
        skip: heuristics::is_skip(role),
    });
    chapters
}
