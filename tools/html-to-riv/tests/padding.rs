use nuxie_html_to_riv::{compile,CompileInput};
fn input(css:&str)->CompileInput {CompileInput {html:"<div id=a></div>".into(),css:css.into(),width:240.,height:160.}}
#[test]
fn public_zero_padding_preserves_prior_bytes_and_maps() {
    let plain=compile(&input("")).unwrap();
    for css in ["#a{padding:0}","#a{padding:0px 0% 0em 0rem}","#a{padding:initial}","#a{padding:unset}","#a{padding-left:0;padding-top:0;padding-right:0;padding-bottom:0}","#a{--p:0;padding:var(--p)}"] {assert_eq!(compile(&input(css)).unwrap(),plain,"{css}");}
}
#[test]
fn public_nonzero_padding_cannot_disappear_in_the_cascade() {
    for css in ["#a{padding:2px}","#never{padding:2px}","#a{padding:2px;padding:0}","#a{--p:2px;padding:var(--p);padding:0}","#never{padding:1em}","#a{padding-top:1%;padding-top:initial}"] {assert_eq!(compile(&input(css)).unwrap_err()[0].code,"unsupported-target-semantics","{css}");}
}
