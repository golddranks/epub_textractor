use std::{fs::File, io::Write, ops::Range, path::Path};

use crate::{
    epub::Epub,
    error::{OrDie, 死},
    heuristics,
};

pub struct Book {
    name: String,
    author: String,
    idxs: Range<usize>,
    files: Vec<String>,
}

pub fn read(fname: &Path) -> Option<Vec<Book>> {
    let Ok(file) = std::fs::read_to_string(fname) else {
        return None;
    };
    let mut books = Vec::new();
    for line in file.lines() {
        let mut fields = line.split(':');
        let name = fields
            .next()
            .or_(死!("Invalid name field in book file"))
            .to_owned();
        let author = fields
            .next()
            .or_(死!("Invalid author field in book file"))
            .to_owned();
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

        books.push(Book {
            name,
            author,
            idxs: idx_start..idx_end,
            files,
        });
    }
    Some(books)
}
pub fn write(books: &[Book], fname: &Path) {
    let mut file = File::create(&fname).or_(死!());
    for book in books {
        write!(file, "{}", book.name).or_(死!());
        write!(file, ":{}", book.author).or_(死!());
        write!(file, ":{}:{}", book.idxs.start, book.idxs.end).or_(死!());
        for fname in &book.files {
            write!(file, ":{}", fname).or_(死!());
        }
        writeln!(file).or_(死!());
    }
}
pub fn generate(epub: &Epub) -> Vec<Book> {
    let name = heuristics::guess_book_name(epub);
    let author = epub.author.to_owned();
    vec![Book {
        name,
        author,
        idxs: 0..epub.texts.len(),
        files: epub.hrefs.iter().map(|(href, _)| href.to_owned()).collect(),
    }]
}
