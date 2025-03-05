use std::{collections::HashMap, error::Error, fs::File, io::Write, path::Path, process::exit};

type Res<T> = Result<T, Box<dyn Error>>;

mod epub;
mod gaiji;
mod txt;
mod yomi;

fn run(epub_fname: &Path) -> Res<()> {
    let txt_fname = epub_fname.with_extension("txt");
    let gaiji_fname = epub_fname.with_extension("gaiji");
    let yomi_fname = epub_fname.with_extension("yomi");

    let mut file = File::open(epub_fname)?;

    let mut gaiji = gaiji::read_gaiji(&gaiji_fname)?.unwrap_or_else(HashMap::new);
    let gaiji_original_size = gaiji.len();

    let epub = epub::extract_contents(&mut file)?;

    let (txt, yomi) = txt::produce_txt_yomi(&mut gaiji, &epub)?;
    let mut txt_file = File::create(&txt_fname)?;
    txt_file.write_all(txt.as_bytes())?;

    if gaiji_original_size != gaiji.len() {
        eprintln!("New gaiji found! Updating/creating the gaiji file.");
        let gaiji_file = File::create(&gaiji_fname)?;
        gaiji::write_gaiji(&gaiji, gaiji_file)?;
    }

    let yomi_file = File::create(&yomi_fname)?;
    yomi::write_yomi(&yomi, yomi_file, &txt)?;

    Ok(())
}

fn main() {
    let epub_fname = std::env::args().nth(1);
    let Some(epub_fname) = epub_fname else {
        eprintln!("Give a filename as a parameter!");
        exit(1);
    };

    if let Err(e) = run(Path::new(&epub_fname)) {
        eprintln!("{e}");
        exit(2);
    }
}
