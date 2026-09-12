//! Public ordinary content owners: native-qualified output, authored semantics,
//! and resource boundaries. Tracked references preserve partial paint evidence.
use nuxie_html_to_riv::{CompileInput, CompileOutput, SourceNode, compile};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scene {
    name: String,
    request: CompileInput,
    riv_hex: String,
    source_map: Vec<SourceNode>,
    control_css: Option<String>,
}
#[derive(Deserialize)]
struct Control { name: String, request: CompileInput, accepted: bool }
fn scenes() -> Vec<Scene> {
    serde_json::from_str(include_str!("fixtures/content-owner/scenes.json")).unwrap()
}
fn bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i+2], 16).unwrap()).collect()
}
fn request(html: String, css: String) -> CompileInput {
    CompileInput { assets: Default::default(),html, css, width:390., height:160.}
}
fn nested(depth: usize) -> String {
    (0..depth).map(|i| format!("<div id=\"n{i}\">" )).collect::<String>() + &"</div>".repeat(depth)
}
fn authored_identities(output: &CompileOutput, count: usize, nested: bool) {
    assert_eq!(output.source_map.len(), count, "helpers must not create source nodes");
    assert_eq!(output.source_map.iter().map(|n| n.object_id).collect::<BTreeSet<_>>().len(), count);
    let identities: BTreeMap<_, _> = output.source_map.iter().map(|n| (n.id.as_str(), n)).collect();
    for i in 0..count {
        let source = identities.get(format!("n{i}").as_str()).unwrap();
        assert_eq!(source.path, if nested { "/0".repeat(i + 1) } else { format!("/{i}") });
    }
}
const PADDED: &str = "div{box-sizing:content-box;width:8px;height:1px;padding:1px}";

#[test]
fn native_qualified_scenes_keep_exact_r3_bytes_source_identities_and_determinism() {
    let scenes = scenes();
    assert_eq!(scenes.len(), 34);
    assert_eq!(scenes.iter().map(|s| &s.name).collect::<BTreeSet<_>>().len(), 34);
    for scene in scenes {
        let result = compile(&scene.request).unwrap_or_else(|e| panic!("{}: {e:?}", scene.name));
        assert_eq!(result.source_map, scene.source_map, "{}: authored map", scene.name);
        assert_eq!(result.riv, bytes(&scene.riv_hex), "{}: complete native-validated ordinary file", scene.name);
        assert_eq!(compile(&scene.request).unwrap(), result, "{}: deterministic repeat", scene.name);
    }
}

#[test]
fn content_owner_inherits_authored_content_dimensions_padding_and_bounds() {
    let mut compared = 0;
    for scene in scenes() {
        let Some(control_css) = scene.control_css else { continue; };
        for (width, height) in [(240.,160.), (390.,200.), (768.,120.)] {
            let mut original = scene.request.clone(); original.width = width; original.height = height;
            let mut explicit = original.clone(); explicit.css = control_css.clone();
            assert_eq!(compile(&original).unwrap(), compile(&explicit).unwrap(), "{}: explicit computed inheritance", scene.name);
        }
        compared += 1;
    }
    assert_eq!(compared, 3);
}

#[test]
fn both_content_owners_constrain_all_twenty_recursive_percentage_controls() {
    let cases: Vec<Control> = serde_json::from_str(include_str!("fixtures/content-owner/bounds.json")).unwrap();
    assert_eq!(cases.len(), 20);
    assert_eq!(cases.iter().filter(|c| c.accepted).count(), 12);
    for case in cases {
        let result = compile(&case.request);
        if case.accepted {
            assert!(result.is_ok(), "{}: {result:?}", case.name);
        } else {
            let errors = result.expect_err(&case.name);
            assert_eq!(errors[0].code, "unsupported-target-semantics", "{}: {errors:?}", case.name);
            assert!(errors[0].message.contains("Resolved percentage size may exceed finite binary32 geometry"), "{}: {errors:?}", case.name);
        }
    }
}

#[test]
fn content_owner_does_not_admit_unqualified_percentage_point_mixtures() {
    #[derive(Deserialize)] struct Rejection {name: String, request: CompileInput}
    let cases: Vec<Rejection> = serde_json::from_str(include_str!("fixtures/content-owner/rejections.json")).unwrap();
    assert_eq!(cases.len(), 5);
    for case in cases {
        let errors = compile(&case.request).expect_err(&case.name);
        assert_eq!(errors[0].code, "unsupported-target-semantics", "{}: {errors:?}", case.name);
    }
}

#[test]
fn content_owners_preserve_128_authored_levels_and_reject_the_first_excess() {
    let output = compile(&request(nested(128), PADDED.into())).unwrap();
    authored_identities(&output, 128, true);
    let errors = compile(&request(nested(129), PADDED.into())).unwrap_err();
    assert_eq!(errors[0].code, "depth-limit");
}

#[test]
fn content_owners_preserve_8192_authored_objects_and_reject_the_first_excess() {
    let html = (0..8192).map(|i| format!("<div id=\"n{i}\"></div>")).collect::<String>();
    let output = compile(&request(html.clone(), PADDED.into())).unwrap();
    authored_identities(&output, 8192, false);
    let errors = compile(&request(html + "<div id=\"n8192\"></div>", PADDED.into())).unwrap_err();
    assert_eq!(errors[0].code, "object-limit");
}

#[test]
fn substantial_inherited_variable_context_preserves_deep_content_owner_output() {
    // 256 KiB of unused selected tokens plus a used inherited custom value.
    // This exercises bounded native/WASM stack and context retention; the
    // no-full-Style-clone requirement also has an independent source audit.
    let variables = (0..64).map(|i| format!("--unused{i}:{};", "x".repeat(4096))).collect::<String>();
    let css = format!("div{{box-sizing:content-box;width:var(--size,8px);height:1px;padding:1px}}#n0{{--size:8px;{variables}}}#n127{{width:inherit;padding:inherit}}");
    let original = request(nested(128), css);
    let explicit = request(nested(128), format!("{PADDED}#n127{{width:inherit;padding:inherit}}"));
    let started = std::time::Instant::now();
    let output = compile(&original).unwrap();
    authored_identities(&output, 128, true);
    assert_eq!(output, compile(&explicit).unwrap());
    assert_eq!(output, compile(&original).unwrap(), "context-bearing repeat");
    eprintln!("content-owner inherited context: 64 x 4096 bytes, 128 levels, two contextual + one literal compile: {:?}", started.elapsed());
}
