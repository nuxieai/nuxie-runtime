//! Translated from runtime/line_break_dictionary_test.cpp at upstream c02b45aa.
#![cfg(any(
    feature = "text-dict-thai",
    feature = "text-dict-lao",
    feature = "text-dict-khmer",
    feature = "text-dict-burmese"
))]
use nuxie_render_api::line_break::{LineBreak, compute_line_breaks};
use nuxie_render_api::line_break_dictionary::{
    line_break_dictionary_edges, line_break_dictionary_for,
};
#[cfg(all(feature = "text-dict-compressed", feature = "text-dict-thai"))]
use nuxie_render_api::line_break_dictionary_builder::build_line_break_dictionary_edges;

fn segments(utf8: &str) -> String {
    let text: Vec<u32> = utf8.chars().map(u32::from).collect();
    let mut out = vec![LineBreak::None; text.len() + 1];
    compute_line_breaks(&text, &mut out);
    let mut result = String::new();
    for (i, cp) in utf8.chars().enumerate() {
        if i > 0 && out[i] != LineBreak::None {
            result.push('|');
        }
        result.push(cp);
    }
    result
}
fn require_words(words: &[&str]) {
    assert_eq!(segments(&words.concat()), words.join("|"));
}

#[cfg(feature = "text-dict-thai")]
#[test]
fn splits_thai_into_words() {
    require_words(&[
        "\u{0E20}\u{0E32}\u{0E29}\u{0E32}",
        "\u{0E44}\u{0E17}\u{0E22}",
        "\u{0E07}\u{0E48}\u{0E32}\u{0E22}",
        "\u{0E19}\u{0E34}\u{0E14}",
        "\u{0E40}\u{0E14}\u{0E35}\u{0E22}\u{0E27}",
    ]);
    require_words(&[
        "\u{0E01}\u{0E23}\u{0E38}\u{0E07}\u{0E40}\u{0E17}\u{0E1E}\u{0E21}\u{0E2B}\u{0E32}\u{0E19}\u{0E04}\u{0E23}",
        "\u{0E40}\u{0E1B}\u{0E47}\u{0E19}",
        "\u{0E40}\u{0E21}\u{0E37}\u{0E2D}\u{0E07}",
        "\u{0E2B}\u{0E25}\u{0E27}\u{0E07}",
    ]);
    require_words(&[
        "\u{0E40}\u{0E14}\u{0E47}\u{0E01}\u{0E46}",
        "\u{0E0A}\u{0E2D}\u{0E1A}",
        "\u{0E01}\u{0E34}\u{0E19}",
        "\u{0E02}\u{0E19}\u{0E21}",
    ]);
    require_words(&[
        "Hello ",
        "\u{0E20}\u{0E32}\u{0E29}\u{0E32}",
        "\u{0E44}\u{0E17}\u{0E22} ",
        "123 ",
        "ok",
    ]);
}

#[cfg(feature = "text-dict-lao")]
#[test]
fn splits_lao_into_words() {
    require_words(&[
        "\u{0E9B}\u{0EB0}\u{0EC0}\u{0E97}\u{0E94}",
        "\u{0EA5}\u{0EB2}\u{0EA7}",
        "\u{0EC0}\u{0E9B}\u{0EB1}\u{0E99}",
        "\u{0E9B}\u{0EB0}\u{0EC0}\u{0E97}\u{0E94}",
        "\u{0E97}\u{0EB5}\u{0EC8}",
        "\u{0EAA}\u{0EA7}\u{0E8D}\u{0E87}\u{0EB2}\u{0EA1}",
    ]);
}

#[cfg(feature = "text-dict-khmer")]
#[test]
fn splits_khmer_into_words() {
    require_words(&[
        "\u{1781}\u{17D2}\u{1789}\u{17BB}\u{17C6}",
        "\u{179F}\u{17D2}\u{179A}\u{179B}\u{17B6}\u{1789}\u{17CB}",
        "\u{1794}\u{17D2}\u{179A}\u{1791}\u{17C1}\u{179F}",
        "\u{1780}\u{1798}\u{17D2}\u{1796}\u{17BB}\u{1787}\u{17B6}",
    ]);
}

#[cfg(feature = "text-dict-burmese")]
#[test]
fn splits_burmese_into_words() {
    require_words(&[
        "\u{1000}\u{103B}\u{103D}\u{1014}\u{103A}\u{1010}\u{1031}\u{102C}\u{103A}",
        "\u{1019}\u{103C}\u{1014}\u{103A}\u{1019}\u{102C}",
        "\u{1005}\u{1000}\u{102C}\u{1038}\u{1015}\u{103C}\u{1031}\u{102C}",
        "\u{1010}\u{1010}\u{103A}",
        "\u{1010}\u{101A}\u{103A}",
    ]);
    require_words(&[
        "\u{1019}\u{1004}\u{103A}\u{1039}\u{1002}\u{101C}\u{102C}",
        "\u{1015}\u{102B}",
    ]);
}

#[cfg(feature = "text-dict-thai")]
#[test]
fn keeps_thai_syllables_whole() {
    require_words(&["\u{0028}\u{0E31}\u{0E20}\u{0E32}\u{0E29}\u{0E32}\u{0029}"]);
    require_words(&[
        "\u{0E16}\u{0E32}",
        "\u{0E08}\u{0E32}\u{0E23}\u{0E22}\u{0E4C}",
    ]);
    require_words(&[
        "\u{0E2A}\u{0E17}\u{0E34}\u{0E07}",
        "\u{0E44}\u{0E0B}\u{0E15}\u{0E4C}\u{0E2F}",
    ]);
    require_words(&[
        "\u{0E01}\u{0E34}\u{0E19}\u{0E14}\u{0E34}\u{0E4C}",
        "\u{0E01}\u{0E34}\u{0E19}",
    ]);
    require_words(&[
        "\u{0E01}\u{0E34}\u{0E19}\u{0E18}\u{0E38}\u{0E4C}",
        "\u{0E14}\u{0E35}",
    ]);
}

#[cfg(feature = "text-dict-burmese")]
#[test]
fn keeps_burmese_final_consonants() {
    require_words(&["\u{1016}\u{1005}\u{103A}"]);
    require_words(&[
        "\u{101B}\u{1004}\u{103A}\u{1016}\u{103D}\u{1032}\u{1037}",
        "\u{1018}\u{1031}\u{102C}\u{103A}\u{1011}\u{102F}\u{1010}\u{103A}",
    ]);
}

#[cfg(feature = "text-dict-thai")]
#[test]
fn never_splits_a_syllable() {
    let result = segments("\u{0E2D}\u{0E20}\u{0E34}\u{0E0D}\u{0E0D}\u{0E32}\u{0E30}");
    assert!(!result.contains("|\u{0E30}"));
    let result = segments("\u{0E01}\u{0E34}\u{0E19}\u{0E40}\u{0E40}\u{0E01}\u{0E34}\u{0E19}");
    assert!(!result.contains("\u{0E40}|"));
}

#[cfg(all(feature = "text-dict-compressed", feature = "text-dict-thai"))]
#[test]
fn inflates_every_deflate_block_type() {
    let stored = [1, 11, 0, 244, 255, 0, 1, 1, 0, 1, 2, 0, 0, 2, 1, 0];
    let fixed = [99, 96, 100, 100, 96, 100, 98, 96, 96, 98, 100, 0, 0];
    let mut tiny = *line_break_dictionary_for(0x0E01).unwrap();
    tiny.words_size = 11;
    tiny.edge_count = 5;
    tiny.edge_checksum = 0x5D692FE7;
    tiny.words = &stored;
    assert_eq!(build_line_break_dictionary_edges(&tiny).len(), 5);
    tiny.words = &fixed;
    assert_eq!(build_line_break_dictionary_edges(&tiny).len(), 5);
}

#[cfg(all(feature = "text-dict-compressed", feature = "text-dict-thai"))]
#[test]
fn rejects_corrupt_word_lists() {
    let thai = line_break_dictionary_for(0x0E01).unwrap();
    let words = thai.words.to_vec();
    let mut seed = 1u32;
    for i in 0..32 {
        let mut broken = words.clone();
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        if i % 2 == 0 {
            broken.truncate(seed as usize % broken.len());
        } else {
            let index = seed as usize % broken.len();
            broken[index] ^= 1u8 << (seed >> 29);
        }
        let mut copy = *thai;
        copy.words = &broken;
        let edges = build_line_break_dictionary_edges(&copy);
        assert!(edges.is_empty() || edges.len() == thai.edge_count as usize);
    }
}

#[test]
fn dictionaries_match_generator_tries() {
    for cp in [0x0E01, 0x0E81, 0x1780, 0x1000] {
        let Some(dictionary) = line_break_dictionary_for(cp) else {
            continue;
        };
        let edges = line_break_dictionary_edges(dictionary);
        assert_eq!(edges.len(), dictionary.edge_count as usize);
        let mut checksum = 2166136261u32;
        for &edge in edges {
            checksum = (checksum ^ edge).wrapping_mul(16777619);
        }
        assert_eq!(checksum, dictionary.edge_checksum);
        assert_eq!(
            line_break_dictionary_edges(dictionary).as_ptr(),
            edges.as_ptr()
        );
    }
}

#[test]
fn leaves_other_scripts_alone() {
    assert_eq!(
        segments("state-of-the-art words"),
        "state-|of-|the-|art |words"
    );
    assert!(line_break_dictionary_for(u32::from('a')).is_none());
    assert!(line_break_dictionary_for(0x0E50).is_none());
    assert!(line_break_dictionary_for(0x0E3F).is_none());
}
