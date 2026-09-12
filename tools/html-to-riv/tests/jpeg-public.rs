//! Public admission for the frozen JPEG qualification corpus. Encoded source
//! bytes are hydrated explicitly; native pixel qualification remains separate.
use nuxie_html_to_riv::{Asset, CompileInput, compile};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    name: String,
    asset_files: BTreeMap<String, String>,
    input: CompileInput,
    expected: Expected,
    observe_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(tag = "outcome", rename_all = "camelCase")]
enum Expected {
    Compile,
    Diagnostic {
        code: String,
        #[serde(rename = "messageIncludes")]
        message_includes: String,
    },
}

fn cases() -> Vec<Case> {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("../validation/public-jpeg-cases.json")).unwrap();
    assert!(!cases.is_empty(), "Keep the complete admission corpus");
    let names: BTreeSet<_> = cases.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names.len(), cases.len(), "Case identities must be unique");
    cases
}

fn request(case: &Case) -> CompileInput {
    let mut input = case.input.clone();
    assert!(
        input.assets.is_empty(),
        "Assets come from explicit fixture files"
    );
    for (key, file) in &case.asset_files {
        let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(file))
            .unwrap_or_else(|error| panic!("{} asset {key} ({file}): {error}", case.name));
        assert!(!bytes.is_empty(), "{} asset {key}", case.name);
        input.assets.insert(key.clone(), Asset::Image { bytes });
    }
    input
}

#[test]
fn public_jpeg_successes_are_deterministic_and_embed_exact_source_bytes() {
    let cases: Vec<_> = cases()
        .into_iter()
        .filter(|c| matches!(c.expected, Expected::Compile))
        .collect();
    let count = cases.len();
    assert!(count > 0, "Keep successful public JPEG controls");
    for case in cases {
        let input = request(&case);
        let output = compile(&input).unwrap_or_else(|d| panic!("{}: {d:?}", case.name));
        assert_eq!(
            output,
            compile(&input).unwrap(),
            "{} deterministic",
            case.name
        );
        assert!(
            output.riv.starts_with(b"RIVE"),
            "{} ordinary file",
            case.name
        );
        let unique_assets: BTreeSet<_> = input
            .assets
            .values()
            .map(|Asset::Image { bytes }| bytes.as_slice())
            .collect();
        for bytes in unique_assets {
            assert_eq!(
                output
                    .riv
                    .windows(bytes.len())
                    .filter(|window| *window == bytes)
                    .count(),
                1,
                "{} must embed each exact encoded asset once without transcoding",
                case.name
            );
        }
        assert_eq!(
            output
                .source_map
                .iter()
                .map(|n| n.id.as_str())
                .collect::<Vec<_>>(),
            case.observe_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            "{} authored identities",
            case.name
        );
        assert!(output.source_map.iter().all(|n| !n.path.is_empty()));
    }
    println!(
        "{count} successful public JPEG/PNG controls checked twice; exact encoded bytes and source IDs retained"
    );
}

#[test]
fn public_jpeg_skinny_boundaries_keep_the_intended_diagnostic() {
    let cases: Vec<_> = cases()
        .into_iter()
        .filter(|c| matches!(c.expected, Expected::Diagnostic { .. }))
        .collect();
    let count = cases.len();
    assert!(count > 0, "Keep diagnostic public JPEG controls");
    for case in cases {
        let input = request(&case);
        let diagnostics = compile(&input).expect_err(&case.name);
        assert_eq!(
            diagnostics,
            compile(&input).unwrap_err(),
            "{} deterministic diagnostic",
            case.name
        );
        let Expected::Diagnostic {
            code,
            message_includes,
        } = &case.expected
        else {
            unreachable!()
        };
        assert_eq!(diagnostics.len(), 1, "{} diagnostic count", case.name);
        let diagnostic = &diagnostics[0];
        assert_eq!(&diagnostic.code, code, "{} diagnostic code", case.name);
        assert!(
            diagnostic.message.contains(message_includes),
            "{} expected {message_includes:?}, received {:?}",
            case.name,
            diagnostic.message
        );
        assert!(
            !diagnostic.source.is_empty(),
            "{} diagnostic source",
            case.name
        );
    }
    println!(
        "{count} narrow subsampled JPEG boundary controls checked twice; expected diagnostic code and message retained"
    );
}
