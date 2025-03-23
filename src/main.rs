use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
    process::exit,
};

use chapters::Chapter;
use epub::{Epub, Meta};
use error::{OrDie, 即死, 死};
use global_str::GlobalStr;

mod chapters;
mod epub;
mod error;
mod gaiji;
mod global_str;
mod heuristics;
mod markov;
mod txt;
mod yomi;

const SEP: char = '\t'; // For file I/O
static EPUB_FNAME: GlobalStr = GlobalStr::new();
static PHASE: GlobalStr = GlobalStr::new();

pub fn prepare(epub_fname: &Path, output_path: &Path) -> (Epub, Vec<Chapter>) {
    let mut file = File::open(epub_fname).or_(死!("failed to open EPUB file"));
    let epub = Epub::new(&mut file);

    let meta_fname = output_path.join("meta.tsv");
    let chapters_fname = output_path.join("chapters.tsv");
    let meta = epub.get_meta();
    meta.write(&meta_fname);
    let chapters = chapters::read(&chapters_fname).unwrap_or_else(|| {
        let chapters = chapters::generate(&epub, &meta);
        eprintln!("No chapters file found. Writing {chapters_fname:?}");
        chapters::write(&chapters, &chapters_fname);
        chapters
    });

    (epub, chapters)
}

pub fn output_book(
    epub: &Epub,
    meta: &Meta,
    chapters: &[Chapter],
    gaiji: &mut HashMap<String, char>,
    output_path: &Path,
) {
    let txt_fname = output_path.join(&meta.title).with_extension("txt");
    let yomi_fname = output_path.join(&meta.title).with_extension("ruby.yomi");

    let (txt, yomi) = txt::produce_txt_yomi(gaiji, epub, chapters);
    let mut txt_file = File::create(&txt_fname).or_(死!());
    txt_file.write_all(txt.as_bytes()).or_(死!());

    let yomi_file = File::create(&yomi_fname).or_(死!());
    yomi::write_yomi(&yomi, yomi_file, &txt);
}

fn main() {
    let epub_fname = std::env::args().nth(1);
    let Some(epub_fname) = epub_fname else {
        eprintln!("Give a filename as a parameter!");
        exit(1);
    };

    EPUB_FNAME.set(&epub_fname);
    PHASE.set("start");

    let epub_fname = Path::new(&epub_fname);
    let output_path = Path::new(epub_fname).with_extension("");
    create_dir_all(&output_path).or_(死!("failed to create output directory"));

    let (_epub, chapters) = prepare(epub_fname, &output_path);

    let gaiji_fname = output_path.join("gaiji.txt");
    let gaiji = gaiji::read(&gaiji_fname).unwrap_or_default();
    let gaiji_orig_size = gaiji.len();

    let books = chapters.chunk_by(|a, b| a.book_name == b.book_name);
    for _chapters in books {
        //output_book(&epub, chapters, &mut gaiji, &output_path); // TODO
    }

    if gaiji_orig_size != gaiji.len() {
        eprintln!("New gaiji found! Updating/creating the gaiji file.");
        gaiji::write_gaiji(&gaiji, &gaiji_fname);
    }
}
