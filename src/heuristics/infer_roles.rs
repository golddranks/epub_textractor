use crate::{
    chapters::Role,
    heuristics::utils::{contains_any_of, contains_numerals},
    markov,
};

type Probs = [f32; 11];
type Feats = [bool; 11];

fn is_contents(name: &str) -> bool {
    contains_any_of(
        name,
        &["目次", "もくじ", "ＣＯＮＴＥＮＴＳ", "contents", "Contents", "CONTENTS", "Ｍｅｎｕ"],
    )
}

fn extract_features(name: &str) -> Feats {
    [
        contains_any_of(name, &["表紙", "表題紙"]),   // cover
        contains_any_of(name, &["紹介", "登場人物"]), // before_extra
        false,                                        // afterword, but no patterns are known yet
        is_contents(name),                            // contents
        contains_any_of(name, &["プロローグ", "序"]), // prologue
        contains_numerals(name) || contains_any_of(name, &["章"]), // main
        contains_any_of(name, &["エピローグ", "終章"]), //epilogue
        contains_any_of(name, &["外伝", "番外編"]),   // bonus_chapter
        contains_any_of(name, &["あとがき", "後書"]), // afterword
        contains_any_of(name, &["付録"]),             // after_extra
        contains_any_of(name, &["奥付"]),             // copyright
    ]
}

const INIT: Probs = [
    0.2,  // Cover
    0.1,  // BeforeExtra
    0.1,  // Foreword
    0.19, // Contents
    0.2,  // Prologue
    0.2,  // Main
    0.02, // Epilogue
    0.02, // BonusChapter
    0.02, // Afterword
    0.02, // AfterExtra
    0.02, // Copyright
];

#[cfg(test)]
const END: Probs = [
    0.01, // Cover
    0.01, // BeforeExtra
    0.01, // Foreword
    0.1,  // Contents
    0.01, // Prologue
    0.2,  // Main
    0.3,  // Epilogue
    0.3,  // BonusChapter
    0.5,  // Afterword
    0.5,  // AfterExtra
    0.9,  // Copyright
];

const TRANS: [Probs; 11] = [
    [0.001, 0.1, 0.05, 0.05, 0.05, 0.7, 0.01, 0.001, 0.018, 0.01, 0.01], // Cover
    [0.001, 0.6, 0.1, 0.1, 0.1, 0.05, 0.01, 0.001, 0.027, 0.01, 0.001],  // BeforeExtra
    [0.001, 0.05, 0.5, 0.2, 0.1, 0.1, 0.02, 0.001, 0.017, 0.01, 0.001],  // Foreword
    [0.001, 0.1, 0.1, 0.1, 0.3, 0.35, 0.01, 0.001, 0.01, 0.027, 0.001],  // Contents
    [0.001, 0.02, 0.02, 0.05, 0.35, 0.5, 0.03, 0.017, 0.001, 0.01, 0.001], // Prologue
    [0.01, 0.01, 0.01, 0.01, 0.01, 0.6, 0.1, 0.1, 0.05, 0.05, 0.05],     // Main
    [0.001, 0.001, 0.001, 0.001, 0.01, 0.1, 0.6, 0.1, 0.1, 0.076, 0.01], // Epilogue
    [0.001, 0.001, 0.001, 0.001, 0.01, 0.046, 0.3, 0.5, 0.1, 0.02, 0.02], // BonusChapter
    [0.001, 0.001, 0.001, 0.001, 0.001, 0.02, 0.1, 0.02, 0.7, 0.1, 0.055], // Afterword
    [0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.41, 0.5],   // AfterExtra
    [0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 0.1, 0.1, 0.1], // Copyright
];

const EMIT: [Probs; 11] = [
    [0.90, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01], // Cover
    [0.01, 0.80, 0.05, 0.03, 0.02, 0.02, 0.02, 0.02, 0.01, 0.01, 0.01], // BeforeExtra
    [0.01, 0.05, 0.80, 0.03, 0.02, 0.02, 0.02, 0.02, 0.01, 0.01, 0.01], // Foreword
    [0.01, 0.05, 0.03, 0.85, 0.02, 0.01, 0.01, 0.01, 0.001, 0.001, 0.001], // Contents
    [0.01, 0.01, 0.01, 0.01, 0.85, 0.10, 0.001, 0.001, 0.001, 0.001, 0.001], // Prologue
    [0.001, 0.001, 0.001, 0.01, 0.05, 0.90, 0.01, 0.02, 0.01, 0.01, 0.01], // Main
    [0.001, 0.001, 0.001, 0.01, 0.01, 0.01, 0.85, 0.10, 0.01, 0.01, 0.01], // Epilogue
    [0.001, 0.001, 0.001, 0.01, 0.05, 0.05, 0.05, 0.80, 0.03, 0.001, 0.01], // BonusChapter
    [0.001, 0.001, 0.001, 0.001, 0.05, 0.05, 0.05, 0.03, 0.80, 0.01, 0.01], // Afterword
    [0.001, 0.01, 0.01, 0.01, 0.03, 0.03, 0.05, 0.05, 0.05, 0.75, 0.001], // AfterExtra
    [0.01, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 0.001, 0.01, 0.97], // Copyright
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
        emit(&[TRUE, false, false, false, false, false, false, false, false, false, false]),
        [0.90, 0.01, 0.01, 0.01, 0.01, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01] // 0th column
    );
    assert_eq!(
        emit(&[false, TRUE, false, false, false, false, false, false, false, false, false]),
        [0.01, 0.80, 0.05, 0.05, 0.01, 0.001, 0.001, 0.001, 0.001, 0.01, 0.001] // 1st column
    );
    assert_eq!(
        emit(&[false, false, false, false, TRUE, false, false, false, false, false, false]),
        [0.01, 0.02, 0.02, 0.02, 0.85, 0.05, 0.01, 0.05, 0.05, 0.03, 0.001] // 4st column
    );
    assert_eq!(
        emit(&[false, false, false, false, false, TRUE, false, false, false, false, false]),
        [0.01, 0.02, 0.02, 0.01, 0.10, 0.90, 0.01, 0.05, 0.05, 0.03, 0.001] // 5th column
    );
    assert_eq!(
        emit(&[false, false, false, false, TRUE, TRUE, false, false, false, false, false]),
        [
            0.0001,
            0.0004,
            0.0004,
            0.0002,
            0.085,
            0.9 * 0.05,
            0.0001,
            0.05 * 0.05,
            0.05 * 0.05,
            0.0009,
            0.001 * 0.001
        ] // products of 4st and 5th column
    );
}

pub fn infer_roles<'a>(names: impl Iterator<Item = &'a str>) -> Vec<Role> {
    let features: Vec<_> = names.map(|name| extract_features(name)).collect();
    dbg!(&features);
    let path = markov::viterbi(&INIT, &TRANS, emit, &features);

    path.into_iter().map(|s| Role::from_num(s)).collect()
}

#[test]
fn assert_sum_unity() {
    fn assert_unity(probs: Probs, end: f32) {
        let mut sorted = probs.clone();
        sorted.sort_by(f32::total_cmp);
        let sum: f32 = sorted.iter().sum();
        if sum + end != 1.0 {
            panic!("{:?} doesn't sum to 1.0 but {}", probs, sum);
        }
    }

    assert_unity(INIT, 0.0);

    assert_unity(TRANS[0], END[0]);
    assert_unity(TRANS[1], END[1]);
    assert_unity(TRANS[2], END[2]);
    assert_unity(TRANS[3], END[3]);
    assert_unity(TRANS[4], END[4]);
    assert_unity(TRANS[5], END[5]);
    assert_unity(TRANS[6], END[6]);
    assert_unity(TRANS[7], END[7]);
    assert_unity(TRANS[8], END[8]);
    assert_unity(TRANS[9], END[9]);
    assert_unity(TRANS[10], END[10]);
}
#[test]
fn test_extract_features() {
    const TRUE: bool = true; // For visual readability in feature vectors below
    assert_eq!(
        extract_features("表紙"),
        [TRUE, false, false, false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("人物紹介"),
        [false, TRUE, false, false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("CONTENTS"),
        [false, false, false, TRUE, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("目次"),
        [false, false, false, TRUE, false, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("【序幕】 独白"),
        [false, false, false, false, TRUE, false, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("１ 始まりの事件"),
        [false, false, false, false, false, TRUE, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("第一章 長距離偵察任務"),
        [false, false, false, false, false, TRUE, false, false, false, false, false]
    );
    assert_eq!(
        extract_features("終章〈お大事に〉"),
        [false, false, false, false, false, TRUE, TRUE, false, false, false, false]
    );
    assert_eq!(
        extract_features("外伝 借りてきた猫"),
        [false, false, false, false, false, false, false, TRUE, false, false, false]
    );
    assert_eq!(
        extract_features("あとがき"),
        [false, false, false, false, false, false, false, false, TRUE, false, false]
    );

    assert_eq!(
        extract_features("あとがき ─クリスといっしょ！─"),
        [false, false, false, false, false, false, false, false, TRUE, false, false]
    );
    assert_eq!(
        extract_features("付録 歴史概略図"),
        [false, false, false, false, false, false, false, false, false, TRUE, false]
    );
    assert_eq!(
        extract_features("奥付"),
        [false, false, false, false, false, false, false, false, false, false, TRUE]
    );
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
        infer_roles(["CONTENTS", "物語"].into_iter()),
        vec![Role::Contents, Role::Main]
    );
    assert_eq!(
        infer_roles(["第一章 長距離偵察任務", "奥付"].into_iter()),
        vec![Role::Main, Role::Copyright]
    );
}

#[test]
fn test_infer_roles_simple_d() {
    assert_eq!(
        infer_roles(["物語", "付録 歴史概略図"].into_iter()),
        vec![Role::Main, Role::AfterExtra]
    );
    assert_eq!(
        infer_roles(["物語", "あとがき"].into_iter()),
        vec![Role::Main, Role::Afterword]
    );
    assert_eq!(
        infer_roles(["物語", "奥付"].into_iter()),
        vec![Role::Main, Role::Copyright]
    );
    assert_eq!(
        infer_roles(["物語", "外伝"].into_iter()),
        vec![Role::Main, Role::BonusChapter]
    );
    assert_eq!(
        infer_roles(["目次", "物語"].into_iter()),
        vec![Role::Contents, Role::Main]
    );
    assert_eq!(
        infer_roles(["【序幕】 独白", "物語"].into_iter()),
        vec![Role::Prologue, Role::Main]
    );
    assert_eq!(
        infer_roles(["人物紹介", "物語"].into_iter()),
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
        infer_roles(["物語", "あとがき クリスと！"].into_iter()),
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
            Role::Main,
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
