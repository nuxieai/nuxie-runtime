//! Handwritten shared vectors from project-runtime-host.md section 3.1 and
//! its settled URL/calendar notes. This table can be copied by other hosts.
use nuxie_runtime::{RuntimeCompiledValueRule, RuntimeRuleValue as V, RuntimeValueRuleKind as R};

#[test]
fn shared_vectors() {
    let cases = vec![
        (
            R::Length {
                minimum: 2,
                maximum: 2,
            },
            V::Text("e\u{301}"),
            None,
            true,
        ),
        (
            R::Length {
                minimum: 2,
                maximum: 2,
            },
            V::Text("😀"),
            None,
            true,
        ),
        (
            R::Length {
                minimum: 1,
                maximum: 1,
            },
            V::Text("😀"),
            None,
            false,
        ),
        (
            R::Pattern("[A-Z]+|[0-9]{2}".into()),
            V::Text("AB"),
            None,
            true,
        ),
        (
            R::Pattern("[A-Z]+|[0-9]{2}".into()),
            V::Text("12"),
            None,
            true,
        ),
        (
            R::Pattern("[A-Z]+|[0-9]{2}".into()),
            V::Text("aAB"),
            None,
            false,
        ),
        (
            R::Pattern(r"[a\/\{\|\}\-]+".into()),
            V::Text("a/{|}-"),
            None,
            true,
        ),
        (
            R::Pattern(r"[a\/\{\|\}\-]+".into()),
            V::Text("$"),
            None,
            false,
        ),
        (
            R::Pattern(r"[\p{ASCII}&&\p{Letter}]+".into()),
            V::Text("ABC"),
            None,
            true,
        ),
        (
            R::Pattern(r"[\p{ASCII}&&\p{Letter}]+".into()),
            V::Text("é"),
            None,
            false,
        ),
        (
            R::Pattern(r"[[a-z]--[aeiou]]+".into()),
            V::Text("bcdf"),
            None,
            true,
        ),
        (
            R::Pattern(r"[[a-z]--[aeiou]]+".into()),
            V::Text("ae"),
            None,
            false,
        ),
        (R::Pattern(r"[\q{ab|cd}]".into()), V::Text("ab"), None, true),
        (
            R::Pattern(r"[\q{ab|cd}]".into()),
            V::Text("ac"),
            None,
            false,
        ),
        // Without the multiline flag JavaScript's $ requires the end of input.
        (R::Pattern("a".into()), V::Text("a\n"), None, false),
        (
            R::TextMinimum("2026-01-01".into()),
            V::Text("2026-01-01"),
            None,
            true,
        ),
        (
            R::TextMinimum("2026-01-01".into()),
            V::Text("2025-12-31"),
            None,
            false,
        ),
        (
            R::TextMaximum("2026-01-01".into()),
            V::Text("2026-01-02"),
            None,
            false,
        ),
        // A supplementary code point starts with a surrogate below U+E000.
        (R::TextMaximum("\u{e000}".into()), V::Text("😀"), None, true),
        (R::NumberMinimum(0.0), V::Number(f32::NAN), None, false),
        (R::NumberMinimum(0.0), V::Number(f32::INFINITY), None, false),
        (
            R::NumberMinimum(0.0),
            V::Number(f32::NEG_INFINITY),
            None,
            false,
        ),
        (R::NumberMinimum(0.0), V::Number(-0.0), None, true),
        (R::NumberMaximum(0.0), V::Number(f32::NAN), None, false),
        (R::NumberMaximum(0.0), V::Number(f32::INFINITY), None, false),
        (
            R::NumberMaximum(0.0),
            V::Number(f32::NEG_INFINITY),
            None,
            false,
        ),
        (R::NumberMaximum(0.0), V::Number(-0.0), None, true),
        (
            R::AllowedValues(vec!["green".into()]),
            V::Enum(Some("green")),
            None,
            true,
        ),
        (
            R::AllowedValues(vec!["green".into()]),
            V::Enum(Some("red")),
            None,
            false,
        ),
        (
            R::AllowedValues(vec!["green".into()]),
            V::Enum(None),
            None,
            false,
        ),
        (
            R::AllowedValues(vec!["green".into()]),
            V::Text("red"),
            None,
            false,
        ),
        (
            R::ItemCount {
                minimum: 1,
                maximum: 2,
            },
            V::List {
                items: 0,
                picked: None,
            },
            None,
            true,
        ),
        (
            R::ItemCount {
                minimum: 1,
                maximum: 2,
            },
            V::List {
                items: 2,
                picked: None,
            },
            None,
            true,
        ),
        (
            R::ItemCount {
                minimum: 1,
                maximum: 2,
            },
            V::List {
                items: 3,
                picked: None,
            },
            None,
            false,
        ),
        (
            R::PickedCount {
                property: "picked".into(),
                minimum: 2,
                maximum: 3,
            },
            V::List {
                items: 5,
                picked: Some(1),
            },
            None,
            false,
        ),
        (
            R::PickedCount {
                property: "picked".into(),
                minimum: 2,
                maximum: 3,
            },
            V::List {
                items: 5,
                picked: Some(2),
            },
            None,
            true,
        ),
        (
            R::PickedCount {
                property: "picked".into(),
                minimum: 2,
                maximum: 3,
            },
            V::List {
                items: 5,
                picked: Some(4),
            },
            None,
            false,
        ),
        (R::Required, V::Scalar, Some(false), false),
        (R::Required, V::Scalar, Some(true), true),
        (R::Required, V::Scalar, None, true),
        (
            R::Required,
            V::List {
                items: 4,
                picked: Some(0),
            },
            None,
            false,
        ),
        (
            R::Required,
            V::List {
                items: 0,
                picked: None,
            },
            None,
            false,
        ),
        (
            R::Required,
            V::List {
                items: 4,
                picked: Some(1),
            },
            None,
            true,
        ),
        (R::Date, V::Text("2024-02-29"), None, true),
        (R::Date, V::Text("2000-02-29"), None, true),
        (R::Date, V::Text("1900-02-29"), None, false),
        (R::Date, V::Text("2026-02-29"), None, false),
        (R::Date, V::Text("2026-04-31"), None, false),
        (R::Date, V::Text("2026-01-01"), None, true),
        (R::Date, V::Text("0000-01-01"), None, false),
        (R::Date, V::Text("2026-1-01"), None, false),
        (R::Date, V::Text("2026-01-00"), None, false),
        (R::Date, V::Text("２０２６-01-01"), None, false),
        (R::Url, V::Text("https://example.org/a?b=c#d"), None, true),
        (R::Url, V::Text("mailto:a@example.org"), None, true),
        (R::Url, V::Text("https://例え.テスト/"), None, true),
        (R::Url, V::Text("/relative"), None, false),
        (R::Url, V::Text("https://[bad]/"), None, false),
    ];
    for (index, (rule, value, marker, expected)) in cases.into_iter().enumerate() {
        let rule = RuntimeCompiledValueRule::compile(rule).unwrap();
        assert_eq!(
            rule.holds(value, marker),
            expected,
            "vector {index}: {rule:?} {value:?}"
        );
    }
}

#[test]
fn only_required_rejects_empty_text() {
    for rule in [
        R::TextMinimum("a".into()),
        R::TextMaximum("a".into()),
        R::AllowedValues(vec!["a".into()]),
        R::Length {
            minimum: 2,
            maximum: 3,
        },
        R::Pattern("[A-Z]+".into()),
        R::Url,
        R::Date,
    ] {
        assert!(
            RuntimeCompiledValueRule::compile(rule)
                .unwrap()
                .holds(V::Text(""), None)
        );
    }
    assert!(
        !RuntimeCompiledValueRule::compile(R::Required)
            .unwrap()
            .holds(V::Text(""), None)
    );
}

#[test]
fn invalid_bounds_are_install_errors() {
    for rule in [
        R::NumberMinimum(f64::NAN),
        R::NumberMaximum(f64::INFINITY),
        R::Length {
            minimum: 2,
            maximum: 1,
        },
        R::ItemCount {
            minimum: 2,
            maximum: 1,
        },
        R::PickedCount {
            property: "picked".into(),
            minimum: 2,
            maximum: 1,
        },
        R::PickedCount {
            property: "".into(),
            minimum: 0,
            maximum: 1,
        },
    ] {
        assert!(RuntimeCompiledValueRule::compile(rule).is_err());
    }
}

#[test]
fn empty_picks_skip_minimum() {
    let rule = RuntimeCompiledValueRule::compile(R::PickedCount {
        property: "picked".into(),
        minimum: 2,
        maximum: 3,
    })
    .unwrap();
    assert!(rule.holds(
        V::List {
            items: 5,
            picked: Some(0)
        },
        None
    ));
}

#[test]
fn decimal_bounds_use_native_precision() {
    for (kind, value) in [
        (R::NumberMaximum(1.1), 1.1f32),
        (R::NumberMinimum(9.99), 9.99f32),
    ] {
        assert!(
            RuntimeCompiledValueRule::compile(kind)
                .unwrap()
                .holds(V::Number(value), None)
        );
    }
}

#[test]
fn uncompilable_patterns_are_rejected() {
    // An unterminated class and a character forbidden unescaped inside a v class.
    for pattern in ["[", "[a|b]"] {
        assert!(
            RuntimeCompiledValueRule::compile(R::Pattern(pattern.into())).is_err(),
            "invalid pattern {pattern:?} must not become an always-passing rule"
        );
    }
}
