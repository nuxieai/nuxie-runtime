//! Explicit historical source/control pairs; never consult the implementation's
//! classifier or infer admission from whether compilation happens to succeed.
use nuxie_html_to_riv::{compile, CompileInput};

pub fn assert_recovery(file: &str, name: &str, request: &CompileInput) -> bool {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../validation/public-value-token-cases.json"
    )).unwrap();
    let matches: Vec<_> = cases.iter().filter(|c| c["historyFile"] == file && c["historyName"] == name).collect();
    assert!(matches.len() <= 1, "duplicate historical control: {file}/{name}");
    let Some(case) = matches.first() else { return false; };
    assert_eq!(case["outcome"], "equivalent");
    assert_eq!(case["html"], request.html, "historical HTML changed: {file}/{name}");
    assert_eq!(case["css"], request.css, "historical CSS changed: {file}/{name}");
    let expected = CompileInput {
        html: request.html.clone(), css: case["literalCss"].as_str().unwrap().into(),
        width: request.width, height: request.height,
    };
    assert_eq!(compile(request).unwrap_or_else(|d| panic!("{file}/{name}: {d:?}")),
        compile(&expected).unwrap_or_else(|d| panic!("{file}/{name} unset: {d:?}")),
        "{file}/{name}: complete bytes/maps must match independently authored unset");
    true
}
