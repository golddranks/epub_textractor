use std::{collections::HashMap, io::Write, path::Path};

use crate::{
    Ctx,
    error::{OptionOrDie, ResultOrDie, 死},
};

pub fn read_gaiji(ctx: &Ctx, gaiji_fname: &Path) -> Option<HashMap<String, char>> {
    let Ok(file) = std::fs::read_to_string(gaiji_fname) else {
        return None;
    };
    let mut gaiji = HashMap::new();
    for line in file.lines() {
        let (src, gaiji_ch) = line
            .split_once(':')
            .or_die(|| 死!(ctx, "Invalid gaiji file: should have : on every line"));

        let gaiji_ch = gaiji_ch.parse().or_die(|e| {
            死!(
                ctx,
                "Invalid gaiji file: should have a single character (codepoint) after : {e}"
            )
        });

        gaiji.insert(src.to_owned(), gaiji_ch);
    }
    Some(gaiji)
}

pub fn write_gaiji(ctx: &Ctx, gaiji: &HashMap<String, char>, mut file: impl Write) {
    for (src, &gaiji_ch) in gaiji {
        writeln!(file, "{src}:{gaiji_ch}").or_die(|e| 死!(ctx, e))
    }
}
