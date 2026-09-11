//! Public bounds semantics; pixel/layout qualification is recorded separately.
use nuxie_html_to_riv::{CompileInput, compile};
fn input(css: &str) -> CompileInput {
    CompileInput { html:"<section id=p><div id=c></div></section>".into(), css:css.into(), width:240., height:160. }
}
fn equivalent(actual: &str, expected: &str) {
    assert_eq!(compile(&input(actual)).unwrap(), compile(&input(expected)).unwrap());
}
#[test]
fn bounds_compute_font_lengths_before_inheritance() {
    equivalent(
        "#p{font-size:20px;min-width:2em;max-width:8rem;min-height:1em;max-height:5em}#c{font-size:40px;min-width:inherit;max-width:inherit;min-height:inherit;max-height:inherit}",
        "#p,#c{min-width:40px;max-width:128px;min-height:20px;max-height:100px}");
    equivalent("#c{min-width:1.5em;max-height:2rem;font-size:24px}", "#c{min-width:36px;max-height:32px}");
}
#[test]
fn percentages_are_inherited_descriptors_and_not_eager_viewport_pixels() {
    equivalent("#p{height:100px;min-width:20%;max-width:80%;min-height:10%;max-height:90%}#c{min-width:inherit;max-width:inherit;min-height:inherit;max-height:inherit}",
        "#p{height:100px}#p,#c{min-width:20%;max-width:80%;min-height:10%;max-height:90%}");
    assert_ne!(compile(&input("#c{min-width:20%}")).unwrap().riv, compile(&input("#c{min-width:48px}")).unwrap().riv);
}
#[test]
fn maxima_reset_to_absent_and_minima_keep_explicit_zero_reset() {
    for keyword in ["none", "initial", "unset", "NONE"] {
        equivalent(&format!("#p{{max-width:10px;max-height:20px}}#c{{max-width:{keyword};max-height:{keyword}}}"),
            "#p{max-width:10px;max-height:20px}");
    }
    equivalent("#c{min-width:inherit;min-height:inherit;max-width:inherit;max-height:inherit}", "");
    equivalent("#c{min-width:0;min-height:0px}", "");
    assert_ne!(compile(&input("#c{max-width:0;max-height:0}")).unwrap().riv, compile(&input("")).unwrap().riv);
}
#[test]
fn conflicting_bounds_remain_independent_until_native_layout() {
    // CSS permits minimum > maximum. Keep both descriptors: native layout
    // qualification verifies that minimum wins, including after resize.
    let css = "#c{min-width:100px;max-width:20px;min-height:50px;max-height:10px}";
    let actual = compile(&input(css)).unwrap();
    assert_ne!(actual.riv, compile(&input("#c{min-width:100px;min-height:50px}")).unwrap().riv);
    assert_eq!(actual, compile(&input(css)).unwrap());
}
#[test]
fn rejects_invalid_bounds_and_overflow() {
    for css in [
        "#c{min-height:none}", "#c{max-width:auto}", "#c{max-height:-1px}",
        "#never{max-width:1000001%}", "#c{min-width:min-content}", "#c{max-height:fit-content}",
        "#c{min-height:NaNpx}", "#c{max-height:1em 2em}",
        "#c{font-size:1000000px;max-height:2em}", "#c{min-width:1000000rem}",
    ] {
        let errors = compile(&input(css)).unwrap_err();
        assert_eq!(errors[0].code, "unsupported-target-semantics", "{css}");
        assert!(!errors[0].source.is_empty(), "{css}");
    }
}
#[test]
fn percentage_height_bounds_do_not_bypass_indefinite_parent_guard() {
    for bound in ["height", "min-height", "max-height"] {
        let errors = compile(&input(&format!("#p{{height:auto;min-height:100px}}#c{{{bound}:20%}}"))).unwrap_err();
        assert!(errors[0].message.contains("auto-height parent"));
        assert!(compile(&input(&format!("#p{{height:100px}}#c{{{bound}:20%}}"))).is_ok());
    }
    assert!(compile(&input("#p{min-height:20%}#c{min-height:inherit}")).is_err());
}

#[test]
fn automatic_minima_are_distinct_from_zero_and_inherit_as_descriptors() {
    for keyword in ["initial", "unset", "auto", "AUTO"] {
        equivalent(&format!("#p{{min-width:{keyword};min-height:{keyword}}}#c{{min-width:inherit;min-height:inherit}}"),
            "#p,#c{min-width:auto;min-height:auto}");
    }
    assert_ne!(compile(&input("#c{min-width:auto;min-height:auto}")).unwrap().riv,
        compile(&input("#c{min-width:0;min-height:0}")).unwrap().riv);
    equivalent("#c{min-width:auto;min-width:0;min-height:initial;min-height:0}", "");
    equivalent("#c{min-width:0!important;min-width:auto;min-height:0!important;min-height:unset}", "");
}
