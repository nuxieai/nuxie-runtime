//! Public semantics only: independently authored unset/literal controls, no
//! compiler-private classifier expectations and no browser-derived dimensions.
use nuxie_html_to_riv::{compile, CompileInput};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    name: String,
    group: String,
    html: String,
    css: String,
    outcome: String,
    literal_css: Option<String>,
    rollback_css: Option<String>,
}

fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!("../validation/public-value-token-cases.json")).unwrap()
}

fn check(groups: &[&str]) {
    let mut failures = Vec::new();
    let mut count = 0;
    for case in cases().into_iter().filter(|c| groups.contains(&c.group.as_str())) {
        count += 1;
        for (width, height) in [(240., 160.), (768., 120.)] {
            let request = |css: &str| CompileInput { assets: Default::default(),
                html: case.html.clone(), css: css.into(), width, height,
            };
            let actual = compile(&request(&case.css));
            match case.outcome.as_str() {
                "diagnostic" => match actual {
                    Ok(_) => failures.push(format!("{} at {width}x{height}: invalid/unsupported value unexpectedly compiled", case.name)),
                    Err(d) => assert!(!d.is_empty() && d.iter().all(|d| !d.code.is_empty() && !d.source.is_empty() && !d.message.is_empty()), "{} empty diagnostic", case.name),
                },
                "equivalent" | "unset-diagnostic" => {
                    let reference = case.literal_css.as_ref().expect("independent literal/unset CSS");
                    let expected = compile(&request(reference));
                    if case.outcome == "unset-diagnostic" {
                        if actual.is_ok() || expected.is_ok() {
                            failures.push(format!("{} at {width}x{height}: actual error={} unset error={}", case.name, actual.is_err(), expected.is_err()));
                        } else {
                            let actual = actual.unwrap_err();
                            let expected = expected.unwrap_err();
                            let actual_semantics: Vec<_> = actual.iter().map(|d| (&d.code, &d.message)).collect();
                            let expected_semantics: Vec<_> = expected.iter().map(|d| (&d.code, &d.message)).collect();
                            if actual_semantics != expected_semantics {
                                failures.push(format!("{} at {width}x{height}: actual diagnostic semantics={actual_semantics:?}; explicit unset={expected_semantics:?}", case.name));
                            }
                        }
                    } else {
                        match (actual, expected) {
                            (Ok(actual), Ok(expected)) => {
                                if actual != expected {
                                    failures.push(format!("{} at {width}x{height}: bytes/maps differ from independent literal/unset", case.name));
                                }
                                assert_eq!(actual, compile(&request(&case.css)).unwrap(), "{} deterministic", case.name);
                                if let Some(rollback) = &case.rollback_css {
                                    let rollback = compile(&request(rollback)).unwrap_or_else(|d| panic!("{} rollback: {d:?}", case.name));
                                    assert_ne!(expected, rollback, "{} unset control must expose cascade rollback", case.name);
                                }
                            }
                            (actual, expected) => failures.push(format!("{} at {width}x{height}: actual={:?}; independent control={:?}", case.name, actual.err(), expected.err())),
                        }
                    }
                }
                other => panic!("unknown outcome {other}"),
            }
        }
    }
    assert!(count > 0, "fixture groups must exist");
    assert!(failures.is_empty(), "{} of {count} cases failed across two viewports:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn non_css_whitespace_and_escaped_token_content_never_manufacture_literal_values() {
    check(&["literal-boundary", "literal-function-boundary"]);
}

#[test]
fn proven_invalid_variable_boundary_values_compute_unset() {
    check(&["variable-boundary"]);
}

#[test]
fn primitive_wrong_types_arity_and_negative_nonnegative_values_compute_unset() {
    check(&["variable-primitive"]);
}

#[test]
fn primitive_wrong_types_remain_diagnostic_when_authored_literally() {
    check(&["literal-primitive"]);
}

#[test]
fn recovery_preserves_cascade_priority_inheritance_shorthands_and_order() {
    check(&["cascade-recovery"]);
}

#[test]
fn historical_invalid_variable_sources_match_explicit_new_unset_controls() {
    check(&["historical-recovery"]);
}

#[test]
fn css_whitespace_comments_and_valid_ascii_escapes_keep_their_semantics() {
    check(&["valid-boundary"]);
}

#[test]
fn unicode_custom_property_names_keep_their_identity() {
    check(&["unicode-custom-name"]);
}

#[test]
fn valid_unsupported_css_resources_and_unclassified_functions_stay_diagnostic() {
    check(&["retained-diagnostic", "unclassified-function"]);
}
