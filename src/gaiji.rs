use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::error::{OrDie, 死};

pub fn read(fname: &Path) -> Option<HashMap<String, char>> {
    let Ok(file) = std::fs::read_to_string(fname) else {
        return None;
    };
    let mut gaiji = HashMap::new();
    for line in file.lines() {
        let (src, gaiji_ch) = line
            .split_once(':')
            .or_(死!("Invalid gaiji file: should have : on every line"));

        let gaiji_ch = gaiji_ch.parse().or_(死!(
            "Invalid gaiji file: should have a single character (codepoint) after :"
        ));

        gaiji.insert(src.to_owned(), gaiji_ch);
    }
    Some(gaiji)
}

pub fn write_gaiji(gaiji: &HashMap<String, char>, fname: &Path) {
    let mut file = File::create(&fname).or_(死!());
    for (src, &gaiji_ch) in gaiji {
        writeln!(file, "{src}:{gaiji_ch}").or_(死!())
    }
}
