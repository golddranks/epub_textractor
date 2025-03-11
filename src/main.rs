use std::{fs::File, io::Write, path::PathBuf, process::exit};

use chapters::Chapter;
use epub::Epub;
use error::{ResultOrDie, 死};
use global_str::GlobalStr;

mod chapters;
mod epub;
mod error;
mod gaiji;
mod global_str;
mod txt;
mod yomi;
mod heuristics;

static EPUB_FNAME: GlobalStr = GlobalStr::new();
static PHASE: GlobalStr = GlobalStr::new();

fn prepare() -> (Epub, Vec<chapters::Chapter>) {
    let epub_fname = PathBuf::from(EPUB_FNAME.get());
    let chapters_fname = epub_fname.with_extension("chaps");

    let mut file = File::open(epub_fname).or_die(|e| 死!("failed to open EPUB file: {e}"));

    let epub = Epub::new(&mut file);

    let chapters = chapters::read_chapters(&chapters_fname).unwrap_or_else(|| {
        eprintln!("No chapters file found. Generating chapters.");
        let chapters = chapters::generate(&epub);
        let chapters_file = File::create(&chapters_fname).or_die(|e| 死!(e));
        chapters::write_chapters(&chapters, chapters_file);
        chapters
    });

    (epub, chapters)
}

fn run(epub: Epub, chapters: impl Iterator<Item = Chapter>) {
    let epub_fname = PathBuf::from(EPUB_FNAME.get());
    let txt_fname = epub_fname.with_extension("txt");
    let gaiji_fname = epub_fname.with_extension("gaiji");
    let yomi_fname = epub_fname.with_extension("yomi");

    let mut gaiji = gaiji::read_gaiji(&gaiji_fname).unwrap_or_default();
    let gaiji_orig_size = gaiji.len();

    let (txt, yomi) = txt::produce_txt_yomi(&mut gaiji, &epub, chapters);
    let mut txt_file = File::create(&txt_fname).or_die(|e| 死!(e));
    txt_file.write_all(txt.as_bytes()).or_die(|e| 死!(e));

    if gaiji_orig_size != gaiji.len() {
        eprintln!("New gaiji found! Updating/creating the gaiji file.");
        let gaiji_file = File::create(&gaiji_fname).or_die(|e| 死!(e));
        gaiji::write_gaiji(&gaiji, gaiji_file);
    }

    let yomi_file = File::create(&yomi_fname).or_die(|e| 死!(e));
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
