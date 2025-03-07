use std::{fs::File, io::Write, path::Path, process::exit};

use error::{ResultOrDie, 死};

mod epub;
mod error;
mod gaiji;
mod txt;
mod yomi;

#[derive(Clone, Copy, Debug)]
struct Ctx {
    epub_fname: &'static str,
    phase: &'static str,
}

fn run(ctx: &mut Ctx) {
    let epub_fname = Path::new(&ctx.epub_fname);
    let txt_fname = epub_fname.with_extension("txt");
    let gaiji_fname = epub_fname.with_extension("gaiji");
    let yomi_fname = epub_fname.with_extension("yomi");

    let mut file = File::open(epub_fname).or_die(|e| 死!(ctx, "failed to open EPUB file: {e}"));

    let mut gaiji = gaiji::read_gaiji(ctx, &gaiji_fname).unwrap_or_default();
    let gaiji_original_size = gaiji.len();

    let epub = epub::extract_contents(ctx, &mut file);

    let (txt, yomi) = txt::produce_txt_yomi(ctx, &mut gaiji, &epub);
    let mut txt_file = File::create(&txt_fname).or_die(|e| 死!(ctx, e));
    txt_file.write_all(txt.as_bytes()).or_die(|e| 死!(ctx, e));

    if gaiji_original_size != gaiji.len() {
        eprintln!("New gaiji found! Updating/creating the gaiji file.");
        let gaiji_file = File::create(&gaiji_fname).or_die(|e| 死!(ctx, e));
        gaiji::write_gaiji(ctx, &gaiji, gaiji_file);
    }

    let yomi_file = File::create(&yomi_fname).or_die(|e| 死!(ctx, e));
    yomi::write_yomi(ctx, &yomi, yomi_file, &txt);
}

fn main() {
    let epub_fname = std::env::args().nth(1);
    let Some(epub_fname) = epub_fname else {
        eprintln!("Give a filename as a parameter!");
        exit(1);
    };

    let mut ctx = Ctx {
        epub_fname: epub_fname.leak(),
        phase: "start",
    };

    run(&mut ctx);
}
