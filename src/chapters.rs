use std::{fmt::Display, fs::File, io::Write, iter::once, ops::Range, path::Path};

use crate::{epub::Epub, error::OrDie, heuristics, 即死, 死, PHASE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Cover, // Cover picture
    BeforeExtra, // Character explanations, maps, drawings etc.
    Foreword, // Foreword
    Contents, // Table of contents
    Prologue, // Prologue
    Main, // Main chapters
    Epilogue, // Epilogue
    BonusChapter, // Bonus content, shor stories etc.
    Afterword, // Afterword, author's thanks etc.
    AfterExtra, // Additional drawings, popular character contest announcements, commericals
    Copyright, // Copyright, publisher info etc.
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s {
            "cover" => Role::Cover,
            "before_extra" => Role::BeforeExtra,
            "foreword" => Role::Foreword,
            "contents" => Role::Contents,
            "prologue" => Role::Prologue,
            "main" => Role::Main,
            "epilogue" => Role::Epilogue,
            "bonus_chapter" => Role::BonusChapter,
            "afterword" => Role::Afterword,
            "after_extra" => Role::AfterExtra,
            "copyright" => Role::Copyright,
            _ => 即死!("Invalid role: {s}"),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Role::Cover => "cover",
            Role::BeforeExtra => "before_extra",
            Role::Foreword => "foreword",
            Role::Contents => "contents",
            Role::Prologue => "prologue",
            Role::Main => "main",
            Role::Epilogue => "epilogue",
            Role::BonusChapter => "bonus_chapter",
            Role::Afterword => "afterword",
            Role::AfterExtra => "after_extra",
            Role::Copyright => "copyright",
        })
    }
}

#[derive(Debug)]
pub struct Chapter {
    pub book_name: String,
    pub chap_name: String,
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
        let book_name = fields
            .next()
            .or_(死!("Invalid book name field in chapters file"))
            .to_owned();
        let chap_name = fields
            .next()
            .or_(死!("Invalid chapter name field in chapters file"))
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
            book_name,
            chap_name,
            idxs: idx_start..idx_end,
            files,
            role: Role::from_str(role),
            skip,
        });
    }
    Some(chapters)
}

pub fn write(chapters: &[Chapter], fname: &Path) {
    let mut file = File::create(fname).or_(死!());
    for chapter in chapters {
        write!(file, "{}", chapter.book_name).or_(死!());
        write!(file, ":{}", chapter.chap_name).or_(死!());
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
    PHASE.set("generate_chapters");
    if heuristics::n_books(epub) > 1 {
        todo!("omnibus");
    }
    let book_name = heuristics::guess_book_name(epub);
    let mut chapters = Vec::new();

    let Epub {
        href_to_spine_idx,
        toc,
        body,
        ..
    } = epub;

    let other_chapters = toc.windows(2).map(|toc_chap| {
        let [(name, toc_href), (_, next_toc_href)] = toc_chap else {
            unreachable!()
        };
        let start_idx = heuristics::get_spine_idx(href_to_spine_idx, toc_href, name);
        let end_idx = heuristics::get_spine_idx(href_to_spine_idx, next_toc_href, name);
        let idxs = start_idx..end_idx;
        (name, idxs)
    });

    let last_toc_chapter = {
        let (name, toc_href) = toc.last().or_(死!("no chapters in TOC? n = {}", toc.len()));
        let start_idx = heuristics::get_spine_idx(href_to_spine_idx, toc_href, name);
        let idxs = start_idx..body.len();
        (name, idxs)
    };

    let all_chapters = other_chapters.chain(once(last_toc_chapter));

    for (name, idxs) in all_chapters {
        let role = heuristics::guess_role(&chapters, name);
        chapters.push(Chapter {
            book_name: book_name.clone(),
            chap_name: name.to_owned(),
            idxs: idxs.clone(),
            files: body
                .get(idxs.clone())
                .or_(死!("the order of files in TOC {:?} doesn't correspond to spine? ({name})", idxs))
                .iter()
                .map(|(href, _)| href)
                .cloned()
                .collect(),
            role,
            skip: heuristics::is_skip(role),
        });
    }
    chapters
}
