//! Font context affects box lengths only; browser/native qualification is separate.
use nuxie_html_to_riv::{CompileInput, compile};

fn equivalent(html: &str, actual: &str, expected: &str) {
    for width in [240., 390., 768.] {
        let input = |css: &str| CompileInput { assets: Default::default(), html: html.into(), css: css.into(), width, height:160. };
        assert_eq!(compile(&input(actual)).unwrap(), compile(&input(expected)).unwrap(), "{actual}");
    }
}

#[test]
fn nested_font_contexts_compute_before_dimensions_and_rem_stays_root_relative() {
    equivalent(
        "<section id=p><div id=c><div id=g></div></div><div id=s></div></section>",
        "#p{width:50%;height:8em;font-size:20px}#c{width:2em;height:1rem;font-size:150%}#g{width:2em;height:.5rem;font-size:.5em}#s{width:2rem;height:1em;font-size:2rem}",
        "#p{width:50%;height:160px}#c{width:60px;height:16px}#g{width:30px;height:8px}#s{width:32px;height:32px}",
    );
}

#[test]
fn font_inheritance_and_css_wide_keywords_use_computed_parent_or_initial() {
    for keyword in ["inherit", "unset", "INHERIT"] {
        equivalent("<div id=p><div id=c></div></div>",
            &format!("#p{{font-size:24px}}#c{{font-size:{keyword};width:2em;height:.5em}}"),
            "#c{width:48px;height:12px}");
    }
    equivalent("<div id=p><div id=c></div></div>",
        "#p{font-size:24px}#c{font-size:initial;width:2em;height:.5em}",
        "#c{width:32px;height:8px}");
    equivalent("<div id=p><div id=c></div></div>",
        "#p{font-size:1.5em}#c{width:2em;height:1em}",
        "#c{width:48px;height:24px}");
}

#[test]
fn inherited_dimensions_copy_absolute_lengths_not_parent_em_factors() {
    equivalent("<div id=p><div id=c></div><div id=s></div></div>",
        "#p{font-size:20px;width:4em;height:4rem}#c{font-size:40px;width:inherit;height:inherit}#s{font-size:40px;width:4em;height:1em}",
        "#p,#c{width:80px;height:64px}#s{width:160px;height:40px}");
}

#[test]
fn cascade_font_size_always_uses_parent_context_not_previous_declaration() {
    equivalent("<div id=p><div id=c class=box style='font-size:4em;width:2em'></div></div>",
        "#p{font-size:20px}.box{font-size:2em}#c{font-size:3em;font-size:150%!important;height:.5em}",
        "#c{width:60px!important;height:15px}");
    equivalent("<div id=c style='width:2em;font-size:1.25em!important'></div>",
        "#c{font-size:3em!important;height:1em}",
        "#c{width:40px!important;height:20px}");
}

#[test]
fn fractional_and_zero_font_contexts_are_valid() {
    equivalent("<div id=p><div id=c></div></div>",
        "#p{font-size:12.5px;width:1.5em;height:2.25em}#c{font-size:0;width:5em;height:1rem}",
        "#p{width:18.75px;height:28.125px}#c{width:0;height:16px}");
    equivalent("<div id=c></div>", "#c{font-size:1000000px;width:1em;height:0em}", "#c{width:1000000px;height:0}");
}

#[test]
fn invalid_values_and_contextual_overflow_fail_with_diagnostics() {
    for css in [
        "#never{font-size:auto}", "#c{font-size:auto;font-size:16px}",
        "#c{width:-1em;width:10px}", "#c{font-size:-1rem;font-size:0!important}", "#never{font-size:-1em}", "#never{font-size:1000001px}",
        "#never{font-size:large}", "#never{font-size:1ex}", "#never{font-size:1em 2em}",
        "#never{width:1000001em}", "#never{height:-1rem}",
        "#c{font-size:1000000px;width:2em}", "#c{font-size:1000000rem}",
        "#p{font-size:1000000px}#c{font-size:200%}",
        "#p{font-size:1000000px}#c{font-size:2em}",
    ] {
        let input = CompileInput { assets: Default::default(), html:"<div id=p><div id=c></div></div>".into(), css:css.into(), width:240., height:160. };
        let errors=compile(&input).unwrap_err();
        assert!(!errors[0].source.is_empty(), "{css}");
        assert_eq!(errors[0].code,"unsupported-target-semantics", "{css}");
    }
}

#[test]
fn font_context_does_not_admit_glyphs_or_bypass_percentage_height_guard() {
    for (html,css) in [
        ("<div>hello</div>", "div{font-size:16px}"),
        ("<div id=p><div id=c></div></div>", "#p{font-size:16px;height:auto}#c{height:50%}"),
    ] {
        assert!(compile(&CompileInput { assets: Default::default(),html:html.into(),css:css.into(),width:240.,height:160.}).is_err());
    }
}
