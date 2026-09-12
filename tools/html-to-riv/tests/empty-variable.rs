use nuxie_html_to_riv::{CompileInput,compile};
use serde::Deserialize;
#[path = "support/value_token_history.rs"] mod value_token_history;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct Case { name:String,html:String,css:String,literal_css:String,#[serde(default="admitted")] accepted:bool }
fn admitted()->bool{true}
fn compare(case:&Case) {
    for (width,height) in [(240.,160.),(390.,200.),(768.,120.)] {
        let input=|css:&str|CompileInput{ assets: Default::default(),html:case.html.clone(),css:css.into(),width,height};
        let actual=compile(&input(&case.css));
        let expected=compile(&input(&case.literal_css));
        if case.accepted {
            let actual=actual.unwrap_or_else(|d|panic!("{}: {d:?}",case.name));
            assert_eq!(actual,expected.unwrap_or_else(|d|panic!("{} literal: {d:?}",case.name)),"{} {width}x{height}",case.name);
            assert_eq!(actual,compile(&input(&case.css)).unwrap(),"{} deterministic",case.name);
        } else {
            assert!(actual.is_err(),"{} unsupported unset must diagnose",case.name);
            assert!(expected.is_err(),"{} control must remain unsupported",case.name);
        }
    }
}

#[test]
fn empty_token_forms_match_unset_for_all_admitted_property_names(){
    let cases:Vec<Case>=serde_json::from_str(include_str!("../validation/public-empty-variable-forms.json")).unwrap();
    assert_eq!(cases.len(),33*7);
    assert_eq!(cases.iter().filter(|c|c.accepted).count(),30*7);
    for case in cases {compare(&case);}
}

#[test]
fn empty_values_preserve_visible_cascade_inheritance_shorthands_and_order(){
    let cases:Vec<Case>=serde_json::from_str(include_str!("../validation/public-empty-variable-cases.json")).unwrap();
    assert_eq!(cases.len(),26);
    for case in cases {compare(&case);}
}

#[test]
fn historical_nonempty_tokens_recover_only_with_explicit_unset_controls(){
    #[derive(Deserialize)]struct Rejection{name:String,html:String,css:String}
    let cases:Vec<Rejection>=serde_json::from_str(include_str!("../validation/public-empty-variable-rejections.json")).unwrap();
    assert_eq!(cases.len(),27);
    for case in cases {
        let request=CompileInput{ assets: Default::default(),html:case.html,css:case.css,width:240.,height:160.};
        if value_token_history::assert_recovery("public-empty-variable-rejections.json",&case.name,&request) { continue; }
        assert!(compile(&request).is_err(),"{}",case.name);
    }
}

#[test]
fn empty_first_component_does_not_hide_excessive_later_expansion(){
    let environment=format!("--empty:;--large:{};","x".repeat(40_000));
    for empty in ["var(--empty)","var(--missing,)"] {
        let request=|suffix:&str|CompileInput{ assets: Default::default(),html:"<div id=a></div>".into(),css:format!("#a{{{environment}width:{empty} {suffix}}}"),width:240.,height:160.};
        assert!(compile(&request("")).is_ok(),"empty control");
        let nonempty=request("var(--large)");
        let unset=CompileInput{ assets: Default::default(),html:nonempty.html.clone(),css:format!("#a{{{environment}width:unset}}"),width:240.,height:160.};
        assert_eq!(compile(&nonempty).unwrap(),compile(&unset).unwrap(),"one bounded expansion has a definite wrong width keyword and computes unset");
        let excess=compile(&request("var(--large) var(--large)")).unwrap_err();
        assert!(excess.iter().any(|d|d.code=="input-limit"),"excess must propagate after empty first component: {excess:?}");
    }
}
