use nuxie_html_to_riv::{compile, CompileInput};

#[test]
fn supported_nesting_compiles_on_the_default_test_stack() {
    let input = CompileInput { html:format!("{}{}", "<div>".repeat(64), "</div>".repeat(64)),
        width:390.,height:320.,..Default::default() };
    assert_eq!(compile(&input).unwrap().source_map.len(),64);
}

#[test]
fn excessive_nesting_reports_a_diagnostic_on_the_default_test_stack() {
    let input = CompileInput { html:format!("{}{}", "<div>".repeat(66), "</div>".repeat(66)),
        width:390.,height:320.,..Default::default() };
    assert_eq!(compile(&input).unwrap_err()[0].code,"input-limit");
}
