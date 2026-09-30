//! line_break_conformance_test.cpp at upstream 02bea09b.
use nuxie_runtime::source::text::line_break::{LineBreak, compute_line_breaks};

#[test]
fn line_break_classes_match_uax14_conformance_data() {
    let fixture = include_str!("../../../fixtures/unicode/LineBreakTest.txt");
    let mut cases = 0;
    let mut failures = 0;
    let mut first_failure = None;
    for line in fixture.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut text = Vec::new();
        let mut expected = Vec::new();
        for token in line.split_ascii_whitespace() {
            match token {
                "÷" => expected.push(true),
                "×" => expected.push(false),
                hex => text.push(u32::from_str_radix(hex, 16).expect("Unicode scalar hex")),
            }
        }
        assert_eq!(expected.len(), text.len() + 1);
        let mut out = vec![LineBreak::None; text.len() + 1];
        compute_line_breaks(&text, &mut out);
        cases += 1;
        let mut ok = true;
        // Upstream intentionally excludes the start-of-text boundary.
        for i in 1..expected.len() {
            if (out[i] != LineBreak::None) != expected[i] {
                ok = false;
            }
        }
        if !ok {
            failures += 1;
            if first_failure.is_none() {
                first_failure = Some(line);
            }
        }
    }
    assert!(cases > 19000);
    assert_eq!(
        failures,
        0,
        "first failing case: {}",
        first_failure.unwrap_or("")
    );
}
