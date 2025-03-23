use std::{fs::File, io::Write, path::Path};

use crate::{
    SEP,
    epub::{Epub, doc},
    error::{OrDie, 死},
    heuristics,
};

pub struct Meta {
    pub asin: Option<String>,
    pub title: String,
    pub author: String,
    pub label: Option<String>,
    pub publisher: String,
    pub pub_date: String,
}

impl Meta {
    pub fn new(epub: &Epub) -> Meta {
        let asin = doc::get_asin(&epub.content).map(|cow| cow.into_owned());
        let title = doc::get_title(&epub.content).to_string();
        let author = doc::get_author(&epub.content).to_string();
        let publisher = doc::get_publisher(&epub.content).to_string();
        let pub_date = doc::get_date(&epub.content).to_string();

        let (book_name, label) = heuristics::parse_book_title(&title);

        Meta {
            asin,
            title: book_name,
            author,
            label,
            publisher,
            pub_date,
        }
    }

    pub fn write(&self, fname: &Path) {
        let mut file = File::create(fname).or_(死!());
        writeln!(file, "asin{SEP}{}", &self.asin.as_deref().unwrap_or("")).or_(死!());
        writeln!(file, "title{SEP}{}", &self.title).or_(死!());
        writeln!(file, "author{SEP}{}", &self.author).or_(死!());
        writeln!(file, "label{SEP}{}", self.label.as_deref().unwrap_or("")).or_(死!());
        writeln!(file, "publisher{SEP}{}", &self.publisher).or_(死!());
        writeln!(file, "pub_date{SEP}{}", &self.pub_date).or_(死!());
    }
}
