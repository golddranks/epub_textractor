#![feature(let_chains)]
use std::{collections::HashMap, fs::read_to_string};

const ROLES: &[&str] = &[
    "cover",
    "before_extra",
    "foreword",
    "contents",
    "prologue",
    "part_title",
    "main",
    "interlude",
    "epilogue",
    "bonus_chapter",
    "afterword",
    "after_extra",
    "copyright",
];

const ALPHA: f32 = 0.1; // for additive / Laplace smoothing
fn smooth_proba(count: i32, total: i32) -> f32 {
    // We'll count the End state also here as a possible transition
    (count as f32 + ALPHA) / (total as f32 + (ROLES.len() + 1) as f32 * ALPHA)
}

fn main() {
    let chapters = read_to_string("aux_data/chapter_role_train.txt").unwrap();
    let mut trans = HashMap::new();
    let mut base = HashMap::new();
    let mut lines = chapters.lines().peekable();
    while let Some(from) = lines.next()
        && let Some(&to) = lines.peek()
    {
        let trans_entry = trans.entry((from, to)).or_insert(0);
        *trans_entry += 1;
        let base_entry = base.entry(from).or_insert(0);
        *base_entry += 1;
    }

    println!("COUNTS:");
    println!("trans");
    for from in ROLES {
        print!("[");
        for to in ROLES {
            let count = trans.get(&(*from, *to)).copied().unwrap_or(0);
            print!("{count}, ");
        }
        println!("],");
    }

    println!("init");
    print!("[");
    for to in ROLES {
        let count = trans.get(&("", *to)).copied().unwrap_or(0);
        print!("{count}, ");
    }
    println!("];");

    println!("end");
    print!("[");
    for from in ROLES {
        let count = trans.get(&(*from, "")).copied().unwrap_or(0);
        print!("{count}, ");
    }
    println!("];");

    println!("PROBABILITIES:");
    println!("trans");
    for from in ROLES {
        print!("[");
        let total = base[from];
        for to in ROLES {
            let count = trans.get(&(*from, *to)).copied().unwrap_or(0);
            let proba = smooth_proba(count, total);
            print!("{proba}, ");
        }
        println!("],");
    }

    println!("init");
    print!("[");
    let total = base[""];
    for to in ROLES {
        let count = trans.get(&("", *to)).copied().unwrap_or(0);
        let proba = smooth_proba(count, total - 1); // -1 because we won't allow "" -> "" transition
        print!("{proba}, ");
    }
    println!("];");

    println!("end");
    print!("[");
    for from in ROLES {
        let total = base[from];
        let count = trans.get(&(*from, "")).copied().unwrap_or(0);
        let proba = smooth_proba(count, total);
        print!("{proba}, ");
    }
    println!("];");
}
