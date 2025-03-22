use crate::{
    chapters::{self, Role},
    error::即死,
    heuristics::utils::{contains_any_of, contains_numerals},
    markov,
};

fn is_cover(name: &str) -> bool {
    contains_any_of(name, &["表紙", "表題紙"])
}

fn is_toc(name: &str) -> bool {
    contains_any_of(
        name,
        &[
            "目次",
            "もくじ",
            "ＣＯＮＴＥＮＴＳ",
            "contents",
            "Contents",
            "CONTENTS",
            "Ｍｅｎｕ",
        ],
    )
}

fn is_before_extra(name: &str) -> bool {
    contains_any_of(name, &["紹介", "登場人物"])
}

fn is_prologue(name: &str) -> bool {
    contains_any_of(name, &["プロローグ", "序"])
}

fn is_main(name: &str) -> bool {
    contains_numerals(name) || contains_any_of(name, &["章"])
}

fn is_epilogue(name: &str) -> bool {
    contains_any_of(name, &["エピローグ", "終章"])
}

fn is_bonus_chapter(name: &str) -> bool {
    contains_any_of(name, &["外伝", "番外編"])
}

fn is_afterword(name: &str) -> bool {
    contains_any_of(name, &["あとがき", "後書"])
}

fn is_after_extra(name: &str) -> bool {
    contains_any_of(name, &["付録"])
}

fn is_copyright(name: &str) -> bool {
    contains_any_of(name, &["奥付"])
}

fn anything_goes(_: &str) -> bool {
    true
}

fn assumed_order(r: Role) -> usize {
    match r {
        Role::Cover => 0,
        Role::BeforeExtra => 1,
        Role::Foreword => 1,
        Role::Contents => 1,
        Role::Prologue => 2,
        Role::Main => 3,
        Role::Epilogue => 4,
        Role::BonusChapter => 5,
        Role::Afterword => 6,
        Role::AfterExtra => 6,
        Role::Copyright => 7,
    }
}

pub fn guess_role(chapters: &[chapters::Chapter], name: &str) -> Role {
    let highest = chapters.last().map(|c| c.role).unwrap_or(Role::Cover);
    let tests = [
        (true, is_cover as fn(&str) -> bool, Role::Cover),
        (true, is_before_extra, Role::BeforeExtra),
        (true, is_toc, Role::Contents),
        (true, is_prologue, Role::Prologue),
        (true, is_epilogue, Role::Epilogue),
        (true, is_afterword, Role::Afterword),
        (true, is_copyright, Role::Copyright),
        (false, anything_goes, Role::Foreword),
        (false, anything_goes, Role::Main),
        (false, anything_goes, Role::BonusChapter),
        (false, anything_goes, Role::AfterExtra),
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

#[derive(Debug, PartialEq, Eq, Default)]
struct Features {
    cover: bool,
    before_extra: bool,
    contents: bool,
    prologue: bool,
    main: bool,
    epilogue: bool,
    bonus_chapter: bool,
    afterword: bool,
    after_extra: bool,
    copyright: bool,
}

fn extract_features(name: &str) -> Features {
    Features {
        cover: is_cover(name),
        before_extra: is_before_extra(name),
        contents: is_toc(name),
        prologue: is_prologue(name),
        main: is_main(name),
        epilogue: is_epilogue(name),
        bonus_chapter: is_bonus_chapter(name),
        afterword: is_afterword(name),
        after_extra: is_after_extra(name),
        copyright: is_copyright(name),
    }
}

pub fn infer_roles(names: &[&str]) -> Vec<Role> {
    let features: Vec<_> = names.iter().map(|name| extract_features(name)).collect();
    let init = [
        0.29, 0.09, 0.1, 0.29, 0.09, 0.09, 0.01, 0.01, 0.01, 0.01, 0.01,
    ];
    let trans = [
        // Cover
        [
            0.001, 0.1, 0.05, 0.05, 0.05, 0.7, 0.01, 0.001, 0.02, 0.01, 0.01,
        ],
        // BeforeExtra
        [
            0.001, 0.6, 0.1, 0.1, 0.1, 0.05, 0.01, 0.001, 0.03, 0.01, 0.001,
        ],
        // Foreword
        [
            0.001, 0.05, 0.5, 0.2, 0.1, 0.1, 0.02, 0.001, 0.02, 0.01, 0.001,
        ],
        // Contents
        [
            0.001, 0.1, 0.1, 0.5, 0.1, 0.15, 0.01, 0.001, 0.01, 0.02, 0.001,
        ],
        // Prologue
        [
            0.001, 0.02, 0.02, 0.05, 0.5, 0.35, 0.03, 0.01, 0.001, 0.01, 0.001,
        ],
        // Main
        [
            0.001, 0.001, 0.001, 0.001, 0.02, 0.9, 0.05, 0.02, 0.001, 0.01, 0.001,
        ],
        // Epilogue
        [
            0.001, 0.001, 0.001, 0.001, 0.01, 0.1, 0.6, 0.1, 0.1, 0.08, 0.01,
        ],
        // BonusChapter
        [
            0.001, 0.001, 0.001, 0.001, 0.01, 0.05, 0.3, 0.5, 0.1, 0.02, 0.02,
        ],
        // Afterword
        [
            0.001, 0.001, 0.001, 0.001, 0.001, 0.02, 0.1, 0.02, 0.7, 0.1, 0.06,
        ],
        // AfterExtra
        [
            0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 0.05, 0.01, 0.1, 0.75, 0.08,
        ],
        // Copyright
        [
            0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 0.1, 0.1, 0.79,
        ],
    ];
    let emit = |feat: &Features| match feat {
        Features { cover: true, .. } => [
            0.90, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01,
        ],
        Features {
            before_extra: true, ..
        } => [
            0.01, 0.80, 0.05, 0.03, 0.02, 0.02, 0.02, 0.02, 0.01, 0.01, 0.01,
        ],
        Features { contents: true, .. } => [
            0.01, 0.05, 0.85, 0.03, 0.02, 0.01, 0.01, 0.01, 0.001, 0.001, 0.001,
        ],
        Features { prologue: true, .. } => [
            0.01, 0.01, 0.01, 0.85, 0.10, 0.01, 0.001, 0.001, 0.001, 0.001, 0.001,
        ],
        Features { main: true, .. } => [
            0.001, 0.001, 0.001, 0.05, 0.90, 0.001, 0.02, 0.01, 0.01, 0.001, 0.01,
        ],
        Features { epilogue: true, .. } => [
            0.001, 0.001, 0.001, 0.01, 0.10, 0.85, 0.01, 0.01, 0.01, 0.001, 0.01,
        ],
        Features {
            bonus_chapter: true,
            ..
        } => [
            0.001, 0.001, 0.001, 0.01, 0.05, 0.05, 0.80, 0.05, 0.03, 0.001, 0.01,
        ],
        Features {
            afterword: true, ..
        } => [
            0.001, 0.001, 0.001, 0.001, 0.05, 0.05, 0.05, 0.80, 0.03, 0.01, 0.01,
        ],
        Features {
            after_extra: true, ..
        } => [
            0.001, 0.01, 0.01, 0.01, 0.03, 0.03, 0.05, 0.05, 0.75, 0.05, 0.001,
        ],
        Features {
            copyright: true, ..
        } => [
            0.01, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 0.001, 0.97, 0.01,
        ],
        _ => [
            0.10, 0.09, 0.09, 0.09, 0.09, 0.09, 0.09, 0.09, 0.09, 0.09, 0.09,
        ],
    };
    let path = markov::viterbi(&init, &trans, emit, &features);

    path.into_iter().map(|s| Role::from_num(s)).collect()
}

#[test]
fn test_extract_features() {
    assert_eq!(
        extract_features("CONTENTS"),
        Features {
            contents: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("第一章 長距離偵察任務"),
        Features {
            main: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("付録 歴史概略図"),
        Features {
            after_extra: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("あとがき"),
        Features {
            afterword: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("奥付"),
        Features {
            copyright: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("外伝 借りてきた猫"),
        Features {
            bonus_chapter: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("目次"),
        Features {
            contents: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("【序幕】 独白"),
        Features {
            prologue: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("人物紹介"),
        Features {
            before_extra: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("終章〈お大事に〉"),
        Features {
            epilogue: true,
            main: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("１ 始まりの事件"),
        Features {
            main: true,
            ..Default::default()
        }
    );

    assert_eq!(
        extract_features("あとがき ─クリスといっしょ！─"),
        Features {
            afterword: true,
            ..Default::default()
        }
    );
}

#[test]
fn test_infer_roles_simple() {
    assert_eq!(
        infer_roles(&["表紙", "物語"]),
        vec![Role::Cover, Role::Main]
    );
    assert_eq!(
        infer_roles(&["CONTENTS", "物語"]),
        vec![Role::Contents, Role::Main]
    );
    assert_eq!(
        infer_roles(&["第一章 長距離偵察任務", "奥付"]),
        vec![Role::Main, Role::Copyright]
    );
    assert_eq!(
        infer_roles(&["付録 歴史概略図", "物語"]),
        vec![Role::BeforeExtra, Role::Main]
    );
    assert_eq!(
        infer_roles(&["物語", "あとがき"]),
        vec![Role::Main, Role::Afterword]
    );
    assert_eq!(
        infer_roles(&["物語", "奥付"]),
        vec![Role::Main, Role::Copyright]
    );
    assert_eq!(
        infer_roles(&["物語", "外伝"]),
        vec![Role::Main, Role::BonusChapter]
    );
    assert_eq!(
        infer_roles(&["目次", "物語"]),
        vec![Role::Contents, Role::Main]
    );
    assert_eq!(
        infer_roles(&["【序幕】 独白", "物語"]),
        vec![Role::Prologue, Role::Main]
    );
    assert_eq!(
        infer_roles(&["人物紹介", "物語"]),
        vec![Role::BeforeExtra, Role::Main]
    );
    assert_eq!(
        infer_roles(&["終章〈お大事に〉", "奥付"]),
        vec![Role::Main, Role::Copyright]
    );
    assert_eq!(
        infer_roles(&["１ 始まりの事件", "エピローグ"]),
        vec![Role::Main, Role::Epilogue]
    );
    assert_eq!(
        infer_roles(&["物語", "あとがき クリスと！"]),
        vec![Role::Main, Role::Afterword]
    );
}

#[cfg(test)]
mod test_data {
    pub const A: &[&str] = &[
        "CONTENTS",
        "第一章 長距離偵察任務",
        "第二章 親善訪問",
        "第三章 完璧な勝利",
        "第四章 再編",
        "第五章 ドードーバード航空戦",
        "第六章 ドアノッカー作戦",
        "付録 歴史概略図",
        "あとがき",
        "奥付",
    ];

    pub const B: &[&str] = &[
        "CONTENTS",
        "第一章 ダキア戦役",
        "第二章 ノルデン Ⅰ",
        "第三章 ノルデン Ⅱ",
        "第四章 ノルデン沖の悪魔",
        "第五章 ラインの悪魔",
        "第六章 火の試練",
        "第七章 前進準備",
        "外伝 借りてきた猫",
        "付録 歴史概略図",
        "あとがき",
        "奥付",
    ];

    pub const C: &[&str] = &[
        "目次",
        "元最強の剣士は、異世界魔法に憧れる １",
        "初めての憧憬",
        "二人だけの帰り道",
        "あとがき",
        "奥付",
    ];

    pub const D: &[&str] = &[
        "CONTENTS",
        "【序幕】 独白",
        "【第一幕】 女子大生と女教授",
        "【？？？】 4/29 22",
        "【第二幕】 女装少年と女子大生",
        "【？？？】観測結果",
        "【第三幕】 女子大生と女装少年",
        "【記録】 7/7 7",
        "【第四幕】 女子大生と女教授と女装少年 １",
        "【？？？】 7/10 14",
        "【第五幕】 女子大生と女教授と女装少年 ２",
        "【？？？】 7/10 19",
        "【第六幕】 女子大生と女教授と女装少年 ３",
        "【？？？】 7/10 22",
        "【終幕】 女子大生と女教授と女装少年のその後",
        "奥付",
    ];

    pub const E: &[&str] = &[
        "人物紹介",
        "目次",
        "１ ここはどこ？",
        "２ ヴァイオリニストは小学生",
        "３ プリマヴェッラ号のコンサート",
        "４ 海に出るつもりじゃなかった",
        "５ 船の中は大騒ぎ",
        "６ 伸びる魔の手",
        "７ 解決への糸口",
        "８ むなしい謎解き",
        "９ もうひとつの肩書き",
        "あとがき ─クリスといっしょ、また！─",
        "奥付",
    ];

    pub const F: &[&str] = &[
        "序章〈魔法医師〉",
        "第一章〈吸血街〉",
        "第二章〈鉄鎚〉",
        "第三章〈妖術司教〉",
        "第四章〈聖人昇天〉",
        "第五章〈悪魔〉",
        "終章〈お大事に〉",
        "奥付",
    ];

    pub const G: &[&str] = &[
        "人物紹介",
        "目次",
        "１ 始まりの事件",
        "２ クリスが来た！",
        "３ 宝石泥棒と小さな落とし物",
        "４ あっちもこっちも謎だらけ",
        "５ 飲みこまれた金の星",
        "６ 宝石泥棒の正体は？",
        "７ 闇にとびこむ",
        "８ 川辺の追跡",
        "９ お礼は手作りの……",
        "あとがき ─クリスといっしょ！─",
        "奥付",
    ];
}

#[test]
fn test_infer_roles_a() {
    assert_eq!(
        infer_roles(test_data::A),
        vec![
            Role::Contents,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::AfterExtra,
            Role::Afterword,
            Role::Copyright
        ]
    );
}

#[test]
fn test_infer_roles_b() {
    assert_eq!(
        infer_roles(test_data::B),
        vec![
            Role::Contents,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::BonusChapter,
            Role::AfterExtra,
            Role::Afterword,
            Role::Copyright
        ]
    );
}

#[test]
fn test_infer_roles_c() {
    assert_eq!(
        infer_roles(test_data::C),
        vec![
            Role::Contents,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Afterword,
            Role::Copyright
        ]
    );
}

#[test]
fn test_infer_roles_d() {
    assert_eq!(
        infer_roles(test_data::D),
        vec![
            Role::Contents,
            Role::Prologue,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Copyright
        ]
    );
}

#[test]
fn test_infer_roles_e() {
    assert_eq!(
        infer_roles(test_data::E),
        vec![
            Role::BeforeExtra,
            Role::Contents,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Afterword,
            Role::Copyright
        ]
    );
}

#[test]
fn test_infer_roles_f() {
    assert_eq!(
        infer_roles(test_data::F),
        vec![
            Role::Prologue,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Epilogue,
            Role::Copyright
        ]
    );
}

#[test]
fn test_infer_roles_g() {
    assert_eq!(
        infer_roles(test_data::G),
        vec![
            Role::BeforeExtra,
            Role::Contents,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Main,
            Role::Afterword,
            Role::Copyright
        ]
    );
}
