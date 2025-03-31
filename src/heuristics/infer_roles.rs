use crate::{
    chapters::Role,
    heuristics::utils::{contains_any_of, contains_numerals},
    markov,
};

use super::utils::normalize_alphabet;

type Probs = [f32; 13];
type Feats = [bool; 13];

fn extract_features(chapter_name: &str) -> Feats {
    let nname = &normalize_alphabet(chapter_name);
    [
        contains_any_of(nname, &["表紙", "表題紙"]),   // cover
        contains_any_of(nname, &["紹介", "登場人物"]), // before_extra
        false,                                         // foreword, but no patterns are known yet
        contains_any_of(nname, &["目次", "もくじ", "content", "menu"]), // contents
        contains_any_of(
            nname,
            &["プロローグ", "prolog", "序", "開", "始", "前", "intro"],
        ), // prologue
        false,                                         // part title, but no patterns are known yet
        contains_numerals(chapter_name)
            || contains_any_of(nname, &["章", "第", "話", "幕", "巻", "本編"]), // main
        contains_any_of(
            nname,
            &["幕間", "閑話", "番外", "間章", "間頁", "interlude", "intermission"],
        ), // interlude
        contains_any_of(
            nname,
            &["エピローグ", "epilog", "終章", "閉", "終", "outro"],
        ), //epilogue
        contains_any_of(
            nname,
            &["外伝", "番外編", "短編", "おまけ", "書き下ろし", "ss"],
        ), // bonus_chapter
        contains_any_of(nname, &["あとがき", "後書", "解説"]), // afterword
        contains_any_of(nname, &["付録", "収録", "特典", "おまけ", "イラスト"]), // after_extra
        contains_any_of(nname, &["奥付"]),             // copyright
    ]
}

#[test]
fn test_extract_features() {
    const TRUE: bool = true; // For visual readability in feature vectors below
    assert_eq!(
        extract_features("表紙"),
        [TRUE, false, false, false, false, false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("人物紹介"),
        [false, TRUE, false, false, false, false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("CONTENTS"),
        [false, false, false, TRUE, false, false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("目次"),
        [false, false, false, TRUE, false, false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("【序幕】 独白"),
        [false, false, false, false, TRUE, false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("１ 始まりの事件"),
        [false, false, false, false, false, false, TRUE, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("第一章 長距離偵察任務"),
        [false, false, false, false, false, false, TRUE, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("終章〈お大事に〉"),
        [false, false, false, false, false, false, TRUE, false, TRUE, false, false, false, false]
    );
    assert_eq!(
        extract_features("外伝 借りてきた猫"),
        [false, false, false, false, false, false, false, false, false, TRUE, false, false, false]
    );
    assert_eq!(
        extract_features("あとがき"),
        [false, false, false, false, false, false, false, false, false, false, TRUE, false, false]
    );

    assert_eq!(
        extract_features("あとがき ─クリスといっしょ！─"),
        [false, false, false, false, false, false, false, false, false, false, TRUE, false, false]
    );
    assert_eq!(
        extract_features("付録 歴史概略図"),
        [false, false, false, false, false, false, false, false, false, false, false, TRUE, false]
    );
    assert_eq!(
        extract_features("奥付"),
        [false, false, false, false, false, false, false, false, false, false, false, false, TRUE]
    );
}

const INIT: Probs = [
    0.015092502,
    0.010223953,
    0.00048685493,
    0.8184032,
    0.07838365,
    0.010223953,
    0.054040898,
    0.00048685493,
    0.00048685493,
    0.00048685493,
    0.00048685493,
    0.00048685493,
    0.015092502,
];

const TRANS: [Probs; 13] = [
    [
        0.022727273,
        0.022727273,
        0.022727273,
        0.4772727,
        0.25,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
    ],
    [
        0.022727273,
        0.022727273,
        0.022727273,
        0.25,
        0.25,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
        0.022727273,
        0.25,
        0.022727273,
    ],
    [
        0.041666664,
        0.041666664,
        0.041666664,
        0.041666664,
        0.041666664,
        0.041666664,
        0.4583333,
        0.041666664,
        0.041666664,
        0.041666664,
        0.041666664,
        0.041666664,
        0.041666664,
    ],
    [
        0.0005733945,
        0.0063073398,
        0.0063073398,
        0.0005733945,
        0.4191514,
        0.0005733945,
        0.5280963,
        0.0063073398,
        0.0063073398,
        0.0005733945,
        0.0005733945,
        0.0063073398,
        0.01777523,
    ],
    [
        0.0010162601,
        0.0010162601,
        0.0010162601,
        0.011178862,
        0.051829267,
        0.011178862,
        0.91565037,
        0.0010162601,
        0.0010162601,
        0.0010162601,
        0.0010162601,
        0.0010162601,
        0.0010162601,
    ],
    [
        0.013513514,
        0.013513514,
        0.013513514,
        0.013513514,
        0.14864865,
        0.013513514,
        0.6891892,
        0.013513514,
        0.013513514,
        0.013513514,
        0.013513514,
        0.013513514,
        0.013513514,
    ],
    [
        0.000074382624,
        0.000074382624,
        0.000074382624,
        0.000074382624,
        0.000074382624,
        0.0015620351,
        0.82200235,
        0.030571258,
        0.07520083,
        0.011975602,
        0.030571258,
        0.00081820885,
        0.013463255,
    ],
    [
        0.0020242915,
        0.0020242915,
        0.0020242915,
        0.0020242915,
        0.0020242915,
        0.022267206,
        0.7510121,
        0.12348177,
        0.08299594,
        0.0020242915,
        0.0020242915,
        0.0020242915,
        0.0020242915,
    ],
    [
        0.0008976661,
        0.0008976661,
        0.0008976661,
        0.0008976661,
        0.0008976661,
        0.0008976661,
        0.009874327,
        0.0008976661,
        0.036804307,
        0.16247755,
        0.48563734,
        0.018850986,
        0.25224417,
    ],
    [
        0.0014836795,
        0.0014836795,
        0.0014836795,
        0.0014836795,
        0.0014836795,
        0.0014836795,
        0.0014836795,
        0.0014836795,
        0.0014836795,
        0.2240356,
        0.2833828,
        0.045994062,
        0.38724035,
    ],
    [
        0.0008375209,
        0.0008375209,
        0.0008375209,
        0.0008375209,
        0.0008375209,
        0.0008375209,
        0.0008375209,
        0.0008375209,
        0.0008375209,
        0.13484088,
        0.00921273,
        0.084589615,
        0.66247904,
    ],
    [
        0.0046728975,
        0.0046728975,
        0.0046728975,
        0.0046728975,
        0.0046728975,
        0.0046728975,
        0.0046728975,
        0.0046728975,
        0.0046728975,
        0.051401872,
        0.1448598,
        0.09813084,
        0.61214954,
    ],
    [
        0.00058685447,
        0.00058685447,
        0.00058685447,
        0.00058685447,
        0.00058685447,
        0.00058685447,
        0.0064553996,
        0.00058685447,
        0.00058685447,
        0.00058685447,
        0.00058685447,
        0.00058685447,
        0.00058685447,
    ],
];

const END: Probs = [
    0.022727273,
    0.022727273,
    0.041666664,
    0.0005733945,
    0.0010162601,
    0.013513514,
    0.013463255,
    0.0020242915,
    0.027827647,
    0.045994062,
    0.10134003,
    0.051401872,
    0.9865024,
];

#[test]
fn assert_sum_unity() {
    fn assert_unity(line: u32, probs: Probs, end: f32) {
        let mut probs = probs.to_vec();
        probs.push(end);
        probs.sort_by(f32::total_cmp);
        let sum: f32 = probs.iter().sum();
        if sum < 1.0_f32.next_down() || sum > 1.0_f32.next_up() {
            panic!(
                "On line {}: {:?} doesn't sum to 1.0 but {}",
                line, probs, sum
            );
        }
    }

    assert_unity(line!(), INIT, 0.0);

    assert_unity(line!(), TRANS[0], END[0]);
    assert_unity(line!(), TRANS[1], END[1]);
    assert_unity(line!(), TRANS[2], END[2]);
    assert_unity(line!(), TRANS[3], END[3]);
    assert_unity(line!(), TRANS[4], END[4]);
    assert_unity(line!(), TRANS[5], END[5]);
    assert_unity(line!(), TRANS[6], END[6]);
    assert_unity(line!(), TRANS[7], END[7]);
    assert_unity(line!(), TRANS[8], END[8]);
    assert_unity(line!(), TRANS[9], END[9]);
    assert_unity(line!(), TRANS[10], END[10]);
}

const EMIT: [Probs; 13] = [
    [0.90, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01], // Cover
    [0.01, 0.80, 0.05, 0.03, 0.02, 0.01, 0.02, 0.01, 0.02, 0.02, 0.01, 0.01, 0.01], // BeforeExtra
    [0.01, 0.05, 0.80, 0.03, 0.02, 0.01, 0.02, 0.01, 0.02, 0.02, 0.01, 0.01, 0.01], // Foreword
    [0.01, 0.05, 0.03, 0.85, 0.02, 0.01, 0.01, 0.01, 0.01, 0.01, 0.001, 0.001, 0.001], // Contents
    [0.01, 0.01, 0.01, 0.01, 0.85, 0.01, 0.10, 0.01, 0.001, 0.001, 0.001, 0.001, 0.001], // Prologue
    [0.001, 0.001, 0.001, 0.01, 0.05, 0.01, 0.90, 0.01, 0.01, 0.02, 0.01, 0.01, 0.01], // PartTitle
    [0.001, 0.001, 0.001, 0.01, 0.05, 0.01, 0.90, 0.01, 0.01, 0.02, 0.01, 0.01, 0.01], // Main
    [0.001, 0.001, 0.001, 0.01, 0.05, 0.01, 0.90, 0.01, 0.01, 0.02, 0.01, 0.01, 0.01], // Interlude
    [0.001, 0.001, 0.001, 0.01, 0.01, 0.01, 0.01, 0.01, 0.85, 0.10, 0.01, 0.01, 0.01], // Epilogue
    [0.001, 0.001, 0.001, 0.01, 0.05, 0.01, 0.05, 0.01, 0.05, 0.80, 0.03, 0.001, 0.01], // BonusChapter
    [0.001, 0.001, 0.001, 0.001, 0.05, 0.01, 0.05, 0.01, 0.05, 0.03, 0.80, 0.01, 0.01], // Afterword
    [0.001, 0.01, 0.01, 0.01, 0.03, 0.03, 0.01, 0.05, 0.01, 0.05, 0.05, 0.75, 0.001], // AfterExtra
    [0.01, 0.001, 0.001, 0.001, 0.001, 0.01, 0.001, 0.01, 0.001, 0.01, 0.001, 0.01, 0.97], // Copyright
];

fn emit(feats: &Feats) -> Probs {
    EMIT.map(|probs| {
        feats
            .iter()
            .zip(probs)
            .filter_map(|(feat, prob)| feat.then_some(prob))
            .product() // the features are considered independent
    })
}

#[test]
fn test_emit() {
    const TRUE: bool = true;

    assert_eq!(
        emit(&[
            TRUE, false, false, false, false, false, false, false, false, false, false, false,
            false
        ]),
        [0.90, 0.01, 0.01, 0.01, 0.01, 0.01, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01] // 0th column
    );
    assert_eq!(
        emit(&[
            false, TRUE, false, false, false, false, false, false, false, false, false, false,
            false
        ]),
        [0.01, 0.80, 0.05, 0.05, 0.01, 0.01, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 0.001] // 1st column
    );
    assert_eq!(
        emit(&[
            false, false, false, false, false, TRUE, false, false, false, false, false, false,
            false
        ]),
        [0.01, 0.02, 0.02, 0.02, 0.85, 0.01, 0.05, 0.01, 0.05, 0.05, 0.05, 0.03, 0.001] // 4st column
    );
    assert_eq!(
        emit(&[
            false, false, false, false, false, false, TRUE, false, false, false, false, false,
            false
        ]),
        [0.01, 0.02, 0.02, 0.01, 0.10, 0.01, 0.90, 0.01, 0.05, 0.05, 0.05, 0.03, 0.001] // 5th column
    );
    assert_eq!(
        emit(&[
            false, false, false, false, false, TRUE, TRUE, false, false, false, false, false, false
        ]),
        [
            0.0001,
            0.0004,
            0.0004,
            0.0002,
            0.0002,
            0.085,
            0.9 * 0.05,
            0.0001,
            0.0001,
            0.05 * 0.05,
            0.05 * 0.05,
            0.0009,
            0.001 * 0.001
        ] // products of 4st and 5th column
    );
}

pub fn infer_roles<'a>(names: impl Iterator<Item = &'a str> + Clone) -> Vec<Role> {
    let features: Vec<_> = names.clone().map(extract_features).collect();
    let path = markov::viterbi(&INIT, &TRANS, &END, emit, &features);

    let roles = path.into_iter().map(Role::from_num).collect::<Vec<_>>();

    for ((name, feats), role) in names.zip(features).zip(roles.clone()) {
        println!("{name}\t{role}\t{feats:?}");
    }

    roles
}

#[test]
fn test_infer_roles_simple_one() {
    assert_eq!(infer_roles(["表紙"].into_iter()), vec![Role::Cover]);
    assert_eq!(infer_roles(["Contents"].into_iter()), vec![Role::Contents]);
    assert_eq!(infer_roles(["Contents"].into_iter()), vec![Role::Contents]);
    assert_eq!(infer_roles(["第一章 長距離"].into_iter()), vec![Role::Main]);
    assert_eq!(infer_roles(["奥付"].into_iter()), vec![Role::Copyright]);
}

#[test]
fn test_infer_roles_simple_two() {
    assert_eq!(
        infer_roles(["表紙", "物語"].into_iter()),
        vec![Role::Cover, Role::Main]
    );
    assert_eq!(
        infer_roles(["CONTENTS", "第一章"].into_iter()),
        vec![Role::Contents, Role::Main]
    );
    assert_eq!(
        infer_roles(["第一章 長距離偵察任務", "奥付"].into_iter()),
        vec![Role::Main, Role::Copyright]
    );
    assert_eq!(
        infer_roles(["第一章", "付録 歴史概略図"].into_iter()),
        vec![Role::Main, Role::AfterExtra]
    );
    assert_eq!(
        infer_roles(["第一章", "あとがき"].into_iter()),
        vec![Role::Main, Role::Afterword]
    );
    assert_eq!(
        infer_roles(["第一章", "奥付"].into_iter()),
        vec![Role::Main, Role::Copyright]
    );
    assert_eq!(
        infer_roles(["第一章", "外伝"].into_iter()),
        vec![Role::Main, Role::BonusChapter]
    );
    assert_eq!(
        infer_roles(["目次", "第一章"].into_iter()),
        vec![Role::Contents, Role::Main]
    );
    assert_eq!(
        infer_roles(["【序幕】 独白", "第一章"].into_iter()),
        vec![Role::Prologue, Role::Main]
    );
    assert_eq!(
        infer_roles(["人物紹介", "第一章"].into_iter()),
        vec![Role::BeforeExtra, Role::Main]
    );
    assert_eq!(
        infer_roles(["終章〈お大事に〉", "奥付"].into_iter()),
        vec![Role::Main, Role::Copyright]
    );
    assert_eq!(
        infer_roles(["１ 始まりの事件", "エピローグ"].into_iter()),
        vec![Role::Main, Role::Epilogue]
    );
    assert_eq!(
        infer_roles(["第一章", "あとがき クリスと！"].into_iter()),
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
        infer_roles(test_data::A.iter().copied()),
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
        infer_roles(test_data::B.iter().copied()),
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
        infer_roles(test_data::C.iter().copied()),
        vec![Role::Contents, Role::Main, Role::Main, Role::Main, Role::Afterword, Role::Copyright]
    );
}

#[test]
fn test_infer_roles_d() {
    assert_eq!(
        infer_roles(test_data::D.iter().copied()),
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
            Role::Epilogue,
            Role::Copyright
        ]
    );
}

#[test]
fn test_infer_roles_e() {
    assert_eq!(
        infer_roles(test_data::E.iter().copied()),
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
        infer_roles(test_data::F.iter().copied()),
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
        infer_roles(test_data::G.iter().copied()),
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
