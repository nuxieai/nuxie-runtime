//! Public receiving-value recovery, with independently authored unset/literal
//! controls. The corpus also preserves valid CSS outside image target admission.
use nuxie_html_to_riv::{Asset, CompileInput, compile};
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    property: String,
    value: String,
    target: String,
}

fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!("../validation/image-variable-recovery-cases.json")).unwrap()
}

fn request(extra: &str) -> CompileInput {
    let mut input = CompileInput {
        html: "<div id=parent><img id=image src=picture></div>".into(),
        css: format!("#parent{{width:200px;height:160px;object-fit:cover;object-position:right bottom;image-rendering:pixelated}}#image{{width:96px;height:64px;object-fit:contain;object-position:left top;image-rendering:auto}}{extra}"),
        width: 240.,
        height: 160.,
        assets: Default::default(),
    };
    input.assets.insert("picture".into(), Asset::Image {
        bytes: include_bytes!("../fixtures/images/ordinary-r1/opaque.png").to_vec(),
    });
    input
}

fn declarations(property: &str, value: &str, selected: &str) -> String {
    format!("#image{{--v:{value};{property}:{selected}}}")
}

#[test]
fn primitive_invalid_image_variables_compute_unset_without_cascade_rollback() {
    let mut failures = Vec::new();
    let rollback = compile(&request("")).unwrap();
    let cases: Vec<_> = cases().into_iter().filter(|c| c.target == "unset").collect();
    assert_eq!(cases.len(), 31);
    for c in cases {
        let actual = compile(&request(&declarations(&c.property, &c.value, "var(--v)")));
        let unset = compile(&request(&declarations(&c.property, &c.value, "unset")))
            .unwrap_or_else(|d| panic!("{} independent unset: {d:?}", c.name));
        assert_ne!(unset, rollback, "{} must expose rollback", c.name);
        match actual {
            Ok(actual) if actual == unset => (),
            other => failures.push(format!("{}: actual diagnostic {:?}, equivalent={}", c.name,
                other.as_ref().err(), other.as_ref().is_ok_and(|a| a == &unset))),
        }
        let literal = compile(&request(&format!("#image{{{}:{}}}", c.property, c.value)));
        assert!(literal.is_err(), "{} invalid literal must diagnose", c.name);
    }
    assert!(failures.is_empty(), "{} recovery failures:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn valid_image_variable_tokens_keep_literal_bytes_and_identities() {
    let cases: Vec<_> = cases().into_iter().filter(|c| c.target == "literal").collect();
    assert_eq!(cases.len(), 16);
    for c in cases {
        let input = request(&declarations(&c.property, &c.value, "var(--v)"));
        let actual = compile(&input).unwrap_or_else(|d| panic!("{} variable: {d:?}", c.name));
        let literal = compile(&request(&declarations(&c.property, &c.value, &c.value)))
            .unwrap_or_else(|d| panic!("{} literal: {d:?}", c.name));
        assert_eq!(actual, literal, "{} exact RIV and map", c.name);
        assert_eq!(actual, compile(&input).unwrap(), "{} deterministic", c.name);
        assert_eq!(actual.source_map.iter().map(|n| (n.id.as_str(), n.path.as_str())).collect::<Vec<_>>(),
            [("parent", "/0"), ("image", "/0/0")]);
    }
}

#[test]
fn valid_unsupported_and_unclassified_image_variables_stay_diagnostic() {
    let cases: Vec<_> = cases().into_iter().filter(|c| c.target == "diagnostic").collect();
    assert_eq!(cases.len(), 24);
    for c in cases {
        for selected in ["var(--v)", c.value.as_str()] {
            let diagnostics = compile(&request(&declarations(&c.property, &c.value, selected)))
                .expect_err(&format!("{} {selected} must remain diagnostic", c.name));
            assert!(!diagnostics.is_empty());
            assert!(diagnostics.iter().all(|d| !d.code.is_empty() && !d.source.is_empty() && !d.message.is_empty()));
        }
    }
}

#[test]
fn image_recovery_preserves_alias_inheritance_fallback_and_importance() {
    for (property, prior) in [("object-fit", "contain"), ("object-position", "left top"), ("image-rendering", "auto")] {
        let inherited = format!("#parent{{--bad:bogus;--alias:var(--bad)}}#image{{{property}:var(--alias,{prior})}}");
        let control = format!("#parent{{--bad:bogus;--alias:var(--bad)}}#image{{{property}:unset}}");
        let actual = compile(&request(&inherited)).unwrap_or_else(|d| panic!("{property}: {d:?}"));
        assert_eq!(actual, compile(&request(&control)).unwrap(), "present invalid alias must not select fallback");
        assert_ne!(actual, compile(&request(&format!("#image{{{property}:{prior}}}"))).unwrap());

        let important = format!("img{{{property}:{prior}!important}}#image{{--v:bogus;{property}:var(--v)!important}}#image{{{property}:{prior}}}");
        let control = important.replace("var(--v)!important", "unset!important");
        assert_eq!(compile(&request(&important)).unwrap(), compile(&request(&control)).unwrap(), "{property} important priority");

        for wide in ["initial", "inherit", "unset"] {
            let fallback = format!("#image{{{property}:var(--missing,{wide})}}");
            let literal = format!("#image{{{property}:{wide}}}");
            assert_eq!(compile(&request(&fallback)).unwrap(), compile(&request(&literal)).unwrap(), "{property} {wide}");
        }
    }
}
