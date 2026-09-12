use nuxie_html_to_riv::{compile, CompileInput};
fn request(css: &str) -> CompileInput {
    CompileInput { html: "<div id='p'><div id='c'></div></div>".into(),
        css: format!("#p{{width:120px;height:80px}}#c{{width:40px;height:20px;background:red}}{css}"),
        width: 240., height: 160., assets: Default::default() }
}
#[test]
fn wrapping_substitutions_preserve_valid_values_and_invalid_values_reset() {
    let wrapped = compile(&request("#p{flex-wrap:wrap;align-content:flex-start}")).unwrap();
    assert_eq!(wrapped, compile(&request("#p{--w:wrap;--a:flex-start;flex-wrap:var(--w);align-content:var(--a)}")).unwrap());
    let nowrap = compile(&request("")).unwrap();
    for value in ["bogus", "12px", "wrap nowrap"] {
        assert_eq!(nowrap, compile(&request(&format!("#p{{--w:{value};flex-wrap:var(--w)}}"))).unwrap());
    }
    // Valid unsupported distributions must not recover to normal as invalid CSS.
    assert!(compile(&request("#p{--a:space-between;align-content:var(--a)}")).is_err());
    assert_eq!(compile(&request("#p{flex-wrap:wrap;--a:bogus;align-content:var(--a)}")).unwrap(), compile(&request("#p{flex-wrap:wrap;align-content:normal}")).unwrap());
}
#[test]
fn explicit_inheritance_and_noninherited_resets_reach_contextual_guards() {
    for reset in ["initial", "unset"] {
        assert!(compile(&request(&format!("#p{{flex-wrap:wrap;align-content:flex-start}}#c{{flex-wrap:{reset}}}"))).is_ok());
    }
    assert!(compile(&request("#p{flex-wrap:wrap;align-content:flex-start}#c{flex-wrap:inherit}")).unwrap_err()[0].message.contains("Nested"));
}
