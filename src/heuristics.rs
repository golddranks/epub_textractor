use crate::{
    chapters::{self, Role},
    epub::Epub,
    error::即死,
};

fn is_index(name: &str) -> bool {
    [
        "表紙",
        "目次",
        "ＣＯＮＴＥＮＴＳ",
        "contents",
        "Contents",
        "CONTENTS",
    ]
    .contains(&name)
}

fn is_afterword(name: &str) -> bool {
    ["あとがき", "後書き", "後書"].contains(&name)
}

fn is_prologue(name: &str) -> bool {
    ["プロローグ"].contains(&name)
}

fn is_epilogue(name: &str) -> bool {
    ["エピローグ"].contains(&name)
}

fn is_cover(name: &str) -> bool {
    ["表紙"].contains(&name)
}

fn is_copyright(name: &str) -> bool {
    ["奥付"].contains(&name)
}

fn anything_goes(_: &str) -> bool {
    true
}

fn assumed_order(r: Role) -> usize {
    match r {
        Role::Cover => 0,
        Role::Index => 1,
        Role::Prologue => 2,
        Role::Main => 3,
        Role::Epilogue => 4,
        Role::BonusChapter => 5,
        Role::Afterword => 6,
        Role::Extra => 7,
        Role::Copyright => 8,
    }
}

pub fn guess_role(chapters: &[chapters::Chapter], name: &str) -> Role {
    let highest = chapters.last().map(|c| c.role).unwrap_or(Role::Cover);
    let tests = [
        (true, is_cover as fn(&str) -> bool, Role::Cover),
        (true, is_index, Role::Index),
        (true, is_prologue, Role::Prologue),
        (true, is_epilogue, Role::Epilogue),
        (true, is_afterword, Role::Afterword),
        (true, is_copyright, Role::Copyright),
        (false, anything_goes, Role::Main),
        (false, anything_goes, Role::BonusChapter),
        (false, anything_goes, Role::Extra),
    ];
    for (reliable, test, role) in tests {
        let matches = test(name);
        if assumed_order(role) < assumed_order(highest) {
            if reliable && matches {
                即死!(
                    "Chapter {name} is out of order with role {}: {:?}",
                    role,
                    chapters
                );
            }
            continue;
        }
        if matches {
            return role;
        }
    }
    即死!("No role found for chapter: {name}");
}

pub fn is_skip(role: Role) -> bool {
    match role {
        Role::Cover | Role::Index | Role::Afterword | Role::Extra | Role::Copyright => true,
        Role::Prologue | Role::Main | Role::Epilogue | Role::BonusChapter => false,
    }
}

pub fn guess_book_name(epub: &Epub) -> String {
    let title = epub.title.to_owned();
    title.replace("【電子書籍限定書き下ろしSS付き】", "")
}
