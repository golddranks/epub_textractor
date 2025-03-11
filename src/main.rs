use std::{fs::File, io::Write, path::PathBuf, process::exit};

use chapters::Chapter;
use epub::Epub;
use error::{OrDie, 即死, 死};
use global_str::GlobalStr;

mod books;
mod chapters;
mod epub;
mod error;
mod gaiji;
mod global_str;
mod heuristics;
mod txt;
mod yomi;

static EPUB_FNAME: GlobalStr = GlobalStr::new();
static PHASE: GlobalStr = GlobalStr::new();

pub fn prepare() -> (Epub, Vec<chapters::Chapter>) {
    let epub_fname = PathBuf::from(EPUB_FNAME.get());
    let mut file = File::open(&epub_fname).or_(死!("failed to open EPUB file"));
    let epub = Epub::new(&mut file);

    let books_fname = epub_fname.with_extension("books");
    let books = books::read(&books_fname).unwrap_or_else(|| {
        eprintln!("No books file found. Generating books.");
        let books = books::generate(&epub);
        books::write(&books, &books_fname);
        books
    });

    let chapters_fname = epub_fname.with_extension("chaps");
    let chapters = chapters::read(&chapters_fname).unwrap_or_else(|| {
        eprintln!("No chapters file found. Generating chapters.");
        let chapters = chapters::generate(&epub);
        chapters::write(&chapters, &chapters_fname);
        chapters
    });

    (epub, chapters)
}

pub fn run(epub: Epub, chapters: impl Iterator<Item = Chapter>) {
    let epub_fname = PathBuf::from(EPUB_FNAME.get());
    let txt_fname = epub_fname.with_extension("txt");
    let gaiji_fname = epub_fname.with_extension("gaiji");
    let yomi_fname = epub_fname.with_extension("yomi");

    let mut gaiji = gaiji::read(&gaiji_fname).unwrap_or_default();
    let gaiji_orig_size = gaiji.len();

    let (txt, yomi) = txt::produce_txt_yomi(&mut gaiji, &epub, chapters);
    let mut txt_file = File::create(&txt_fname).or_(死!());
    txt_file.write_all(txt.as_bytes()).or_(死!());

    if gaiji_orig_size != gaiji.len() {
        eprintln!("New gaiji found! Updating/creating the gaiji file.");
        gaiji::write_gaiji(&gaiji, &gaiji_fname);
    }

    let yomi_file = File::create(&yomi_fname).or_(死!());
    yomi::write_yomi(&yomi, yomi_file, &txt);
}

fn main() {
    let epub_fname = std::env::args().nth(1);
    let Some(epub_fname) = epub_fname else {
        eprintln!("Give a filename as a parameter!");
        exit(1);
    };

    EPUB_FNAME.set(epub_fname);
    PHASE.set("start");

    let (epub, chapters) = prepare();
    let chapters = chapters.into_iter().filter(|c| !c.skip);
    //run(epub, chapters);
}
