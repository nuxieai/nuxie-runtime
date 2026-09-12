use nuxie_html_to_riv::{compile,CompileInput};
fn input(css:&str)->CompileInput {CompileInput {html:"<div id=a></div>".into(),css:css.into(),width:240.,height:160.}}
#[test]
fn public_zero_padding_preserves_prior_bytes_and_maps() {
    let plain=compile(&input("")).unwrap();
    for css in ["#a{padding:0}","#a{padding:0px 0% 0em 0rem}","#a{padding:initial}","#a{padding:unset}","#a{padding-left:0;padding-top:0;padding-right:0;padding-bottom:0}","#a{--p:0;padding:var(--p)}"] {assert_eq!(compile(&input(css)).unwrap(),plain,"{css}");}
}
#[test]
fn public_padding_preserves_strict_unsupported_literal_diagnostics() {
    for css in ["#a{padding:-2px}","#never{padding:auto}","#a{padding:calc(2px);padding:0}","#a{--p:-2px;padding:var(--p);padding:0}","#never{padding:1fr}"] {assert!(compile(&input(css)).is_err(),"{css}");}
}
#[test]
fn public_nonzero_padding_admits_ordinary_border_box_profiles() {
    for css in ["#a{padding:2px;box-sizing:content-box}","#a{padding:2px}","#a{width:1px;height:1px;padding:10px}","#a{padding:1% 2% 3% 4%}","#a{padding:2px;padding:0}","#a{--p:2px;padding:var(--p)}","#a{width:auto;height:auto;align-self:flex-start;padding:2%}"] {assert!(compile(&input(css)).is_ok(),"{css}");}
    for css in ["#a{padding:2px;align-self:center}","#a{padding:2px;align-self:baseline}","#a{padding:2px;flex:1 1 0px}"] {assert!(compile(&input(css)).is_err(),"{css}");}
}
#[test]
fn padding_numeric_guards_check_floors_and_sums_without_baking_viewport() {
    let mut request=CompileInput {html:format!("{}{}",(0..9).map(|i|format!("<div id=n{i}>")).collect::<String>(),"</div>".repeat(9)),css:"div{width:1000000%;height:1px;padding:1px}".into(),width:1.,height:32.};
    assert!(compile(&request).is_err());
    request.css.push_str("#n0{width:0px}");assert!(compile(&request).is_ok());
    request.css="div{width:1000000%;height:1px}#n8{width:1px;padding-left:10500%;padding-right:10500%;max-width:1px}".into();
    assert!(compile(&request).is_err()); // each side finite; sum/floor is not
    request.css.push_str("#n8{padding-right:0}");assert!(compile(&request).is_ok());
}
#[test]
fn percentage_padding_with_unknown_intrinsic_containing_width_diagnoses() {
    let mut request=input("#p{width:auto;align-self:flex-start}#a{padding:10%}");
    request.html="<div id=p><div id=a></div></div>".into();
    assert!(compile(&request).is_err());
    request.css.push_str("#p{width:100px}");assert!(compile(&request).is_ok());
}
