//! Ordinary-file alignment candidates. Native/browser qualification is separate.
use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};
fn input(css:&str)->CompileInput {CompileInput{html:"<div id=p><div id=a><div id=b></div></div><div id=c></div></div>".into(),css:css.into(),width:240.,height:160.}}
fn scene(css:&str)->CompileOutput {compile(&input(css)).unwrap()}
#[test]
fn auto_stretch_and_css_wide_defaults_preserve_existing_bytes() {
    for value in ["auto","stretch","initial","unset","inherit"] {
        assert_eq!(scene(&format!("div{{align-self:{value}}}")),scene(""));
    }
}
#[test]
fn alignment_inherits_final_parent_value_only_when_requested() {
    assert_eq!(scene("#p{align-self:flex-end;align-self:center}#a{align-self:inherit}"),scene("#p,#a{align-self:center}"));
    assert_eq!(scene("#p{align-self:center}#a{align-self:unset}"),scene("#p{align-self:center}"));
    assert_eq!(scene("#p{align-self:center}#a{align-self:initial}"),scene("#p{align-self:center}"));
}
#[test]
fn wrappers_preserve_authored_identity_and_selectors() {
    for direction in ["row","row-reverse","column","column-reverse"] {
        let css=format!("#p{{flex-direction:{direction}}}#a{{align-self:center;order:1}}#c{{align-self:flex-end}}#p>div:first-child{{background:red}}");
        let actual=scene(&css);
        assert_eq!(actual.source_map.iter().map(|n|n.id.as_str()).collect::<Vec<_>>(),["p","a","b","c"]);
        assert_eq!(actual.source_map[1].path,"/0/0");
        assert_eq!(actual.source_map[2].path,"/0/0/0");
        assert_eq!(actual,scene(&css));
    }
}
#[test]
fn variables_cascade_and_font_relative_bounds_compose() {
    assert_eq!(scene("#p{--align:center;font-size:20px}#a{align-self:var(--align)!important;align-self:flex-start;width:2em;min-height:1rem;max-height:3em}"),
        scene("#p{font-size:20px}#a{align-self:center;width:40px;min-height:16px;max-height:60px}"));
    assert_ne!(scene("#a{align-self:flex-start}").riv,scene("").riv);
    assert_ne!(scene("#a{align-self:center}").riv,scene("#a{align-self:flex-end}").riv);
}
#[test]
fn unresolved_alignments_and_invalid_losers_still_reject() {
    for value in ["baseline","first baseline","last baseline","normal","safe center","self-start","left","center end"] {
        for css in [format!("#never{{align-self:{value}}}"),format!("#a{{--v:{value};align-self:var(--v);align-self:auto}}")] {
            assert!(compile(&input(&css)).is_err(),"{css}");
        }
    }
}
#[test]
fn percentage_height_guard_still_uses_authored_parent_context() {
    assert!(compile(&input("#a{align-self:center;height:50%}")).is_err());
    assert!(compile(&input("#p{height:100px}#a{align-self:center;height:50%}")).is_ok());
}
