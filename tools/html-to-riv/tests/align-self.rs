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
fn invalid_alignment_literals_reject_and_variable_losers_compute_unset() {
    for value in ["last baseline extra","safe stretch","unsafe normal","safe baseline","left","center end"] {
        let literal=format!("#never{{align-self:{value}}}");
        assert!(compile(&input(&literal)).is_err(),"{literal}");
        let variable=format!("#a{{--v:{value};align-self:var(--v);align-self:auto}}");
        // The current Alignment draft and pinned Chrome disagree on overflow-
        // position with normal; retain the unclassified target diagnostic.
        if value=="unsafe normal" {
            assert!(compile(&input(&variable)).is_err(),"{variable}");
            continue;
        }
        assert_eq!(scene(&variable),
            scene(&format!("#a{{--v:{value};align-self:unset;align-self:auto}}")),"{value}");
    }
}
#[test]
fn percentage_height_guard_still_uses_authored_parent_context() {
    assert!(compile(&input("#a{align-self:center;height:50%}")).is_err());
    assert!(compile(&input("#p{height:100px}#a{align-self:center;height:50%}")).is_ok());
}
#[test]
fn logical_aliases_and_unsafe_forms_lower_in_the_current_ltr_profile() {
    for (actual, expected) in [("normal","stretch"),("start","flex-start"),("self-start","flex-start"),("end","flex-end"),("self-end","flex-end"),("unsafe center","center"),("unsafe flex-end","flex-end"),("unsafe end","flex-end"),("unsafe self-end","flex-end"),("safe start","flex-start"),("safe self-start","flex-start")] {
        for direction in ["row","row-reverse","column","column-reverse"] {
            assert_eq!(scene(&format!("#p{{flex-direction:{direction}}}#a{{align-self:{actual}}}")),
                scene(&format!("#p{{flex-direction:{direction}}}#a{{align-self:{expected}}}")));
        }
    }
}
#[test]
fn safe_preferences_survive_cascade_variables_and_inheritance() {
    for value in ["safe center","safe end","safe flex-end","safe self-end"] {
        assert_eq!(scene(&format!("#p{{--v:{value};align-self:var(--v)}}#a{{align-self:inherit!important;align-self:unsafe center}}")),
            scene(&format!("#p,#a{{align-self:{value}}}")));
        let actual=scene(&format!("#a{{align-self:{value}}}"));
        assert_eq!(actual.source_map.iter().map(|n|(&n.id,&n.path)).collect::<Vec<_>>(),
            scene("#a{align-self:center}").source_map.iter().map(|n|(&n.id,&n.path)).collect::<Vec<_>>());
    }
    assert_ne!(scene("#a{align-self:safe center}").riv,scene("#a{align-self:unsafe center}").riv);
    assert_ne!(scene("#a{align-self:safe end}").riv,scene("#a{align-self:unsafe end}").riv);
    assert_eq!(scene("#p{align-self:safe center}#a{align-self:unset}"),scene("#p{align-self:safe center}"));
}
#[test]
fn malformed_overflow_alignment_is_not_silently_normalized() {
    for value in ["safe","unsafe","center safe","safe unsafe center","safe center end","unsafe auto","safe normal","unsafe stretch","safe inherit","unsafe initial","safe unset","safe last baseline"] {
        let literal=format!("#a{{align-self:{value}}}");
        assert!(compile(&input(&literal)).is_err(),"{literal}");
        let variable=format!("#a{{--v:{value};align-self:var(--v);align-self:auto}}");
        if value=="safe normal" {
            assert!(compile(&input(&variable)).is_err(),"{variable}");
            continue;
        }
        assert_eq!(scene(&variable),
            scene(&format!("#a{{--v:{value};align-self:unset;align-self:auto}}")),"{value}");
    }
}
