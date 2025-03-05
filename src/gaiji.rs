use std::{collections::HashMap, error::Error, fmt::Display, io::Write, path::Path};

use crate::Res;

#[derive(Debug, Clone, Copy)]
enum MyError {
    InvalidGaijiFile,
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::InvalidGaijiFile => f.write_str("Invalid gaiji file!"),
        }
    }
}

impl Error for MyError {}

pub fn read_gaiji(gaiji_fname: &Path) -> Res<Option<HashMap<String, char>>> {
    let Ok(file) = std::fs::read_to_string(gaiji_fname) else {
        return Ok(None);
    };
    let mut gaiji = HashMap::new();
    for line in file.lines() {
        let Some((src, gaiji_ch)) = line.split_once(':') else {
            return Err(MyError::InvalidGaijiFile)?;
        };
        gaiji.insert(src.to_owned(), gaiji_ch.parse()?);
    }
    Ok(Some(gaiji))
}

pub fn write_gaiji(gaiji: &HashMap<String, char>, mut file: impl Write) -> Res<()> {
    for (src, &gaiji_ch) in gaiji {
        writeln!(file, "{src}:{gaiji_ch}")?;
    }
    Ok(())
}
