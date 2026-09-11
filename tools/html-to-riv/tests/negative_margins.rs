use nuxie_html_to_riv::{compile, CompileInput};
fn input(css: &str) -> CompileInput { CompileInput { html: "<div id=root><div id=a></div></div>".into(), css: css.into(), width:390., height:320., ..Default::default() } }
#[test]
fn signed_margin_shorthand_and_units_compile_equivalently() {
    for (a,b) in [
        ("margin:-5px -2% auto 3px", "margin-top:-5px;margin-right:-2%;margin-bottom:auto;margin-left:3px"),
        ("font-size:20px;margin:-1em", "font-size:20px;margin:-20px"),
        ("margin:-1rem", "margin:-16px"),
        ("--m:-5px -2%;margin:var(--m)", "margin:-5px -2%"),
    ] { let a=compile(&input(&format!("#a{{{a}}}"))).unwrap(); let b=compile(&input(&format!("#a{{{b}}}"))).unwrap(); assert_eq!(a.riv,b.riv); assert_eq!(a.runtime_requirements,b.runtime_requirements); }
}
#[test]
fn signed_margin_bounds_do_not_allow_negative_padding_or_dimensions() {
    for v in ["-1000000px", "-10000%", "-0px", "-0%"] { compile(&input(&format!("#a{{margin:{v}}}"))).unwrap(); }
    for declaration in ["margin:-1000001px", "margin:-10001%", "margin:-1e30px", "margin:-auto", "padding:-1px", "padding:-1%", "width:-1px", "gap:-1px"] { assert!(compile(&input(&format!("#a{{{declaration}}}"))).is_err(),"{declaration}"); }
}
