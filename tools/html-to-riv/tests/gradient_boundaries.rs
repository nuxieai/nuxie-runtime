use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};

fn input(html: &str, css: &str) -> CompileInput {
    CompileInput {
        html: html.into(), css: css.into(), width: 390., height: 320.,
        ..Default::default()
    }
}

fn target(output: &CompileOutput, id: &str) -> u32 {
    output.source_map.iter().find(|node| node.id == id).unwrap().object_id
}

#[test]
fn gradient_emission_preserves_the_accepted_depth_boundary_on_default_stack() {
    for levels in [65, 66] {
        let html = format!("{}{}", "<div>".repeat(levels), "</div>".repeat(levels));
        let result = compile(&input(&html, "div{background:linear-gradient(red,blue)}"));
        if levels == 65 {
            let output = result.unwrap();
            assert_eq!(output.source_map.len(), levels);
            assert_eq!(output.runtime_requirements.layout_linear_gradients.len(), levels);
            for (node, gradient) in output.source_map.iter()
                .zip(&output.runtime_requirements.layout_linear_gradients) {
                assert_eq!(node.object_id, gradient.object_id);
                assert!(output.runtime_requirements.layout_pixel_bounds.contains(&node.object_id));
            }
        } else {
            assert_eq!(result.unwrap_err()[0].code, "input-limit");
        }
    }
}

#[test]
fn gradients_do_not_reduce_the_element_count_budget() {
    for count in [2048, 2049] {
        let result = compile(&input(&"<div></div>".repeat(count),
            "div{background:linear-gradient(red,blue)}"));
        if count == 2048 {
            let output = result.unwrap();
            assert_eq!(output.source_map.len(), count);
            assert_eq!(output.runtime_requirements.layout_linear_gradients.len(), count);
            let ids = output.runtime_requirements.layout_linear_gradients.iter()
                .map(|gradient| gradient.object_id).collect::<std::collections::BTreeSet<_>>();
            assert_eq!(ids.len(), count);
        } else {
            assert_eq!(result.unwrap_err()[0].code, "input-limit");
        }
    }
}

#[test]
fn hidden_gradient_subtrees_keep_source_identity_and_existing_visibility_encoding() {
    let html = "<div id=hidden><div id=child></div></div><div id=visible></div>";
    let base = "#hidden{display:none}#child{display:flex}";
    let baseline = compile(&input(html, base)).unwrap();
    let output = compile(&input(html, &format!(
        "{base}div{{background-image:linear-gradient(red,blue)}}"
    ))).unwrap();
    // Gradient transport must not rewrite the Rive display/hidden flags, including
    // a child whose own display:flex cannot escape its display:none ancestor.
    assert_eq!(output.riv, baseline.riv);
    assert_eq!(output.runtime_requirements.layout_linear_gradients.len(), 3);
    for id in ["hidden", "child", "visible"] {
        let object_id = target(&output, id);
        assert_eq!(object_id, target(&baseline, id));
        assert!(output.runtime_requirements.layout_linear_gradients.iter()
            .any(|gradient| gradient.object_id == object_id));
    }
    assert_ne!(output.riv, compile(&input(html,
        "div{background-image:linear-gradient(red,blue)}")).unwrap().riv);
}

#[test]
fn reordered_gradient_targets_follow_authored_identity_and_reject_duplicates() {
    let output = compile(&input("<div id=a></div><div id=b></div>",
        "#a{order:2;background:linear-gradient(red,blue)}#b{order:-1;background:linear-gradient(lime,black)}"
    )).unwrap();
    assert_eq!(output.source_map.iter().map(|node| node.id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
    assert!(target(&output, "b") < target(&output, "a"));
    for (id, color) in [("a", 0xffff0000), ("b", 0xff00ff00)] {
        let gradient = output.runtime_requirements.layout_linear_gradients.iter()
            .find(|gradient| gradient.object_id == target(&output, id)).unwrap();
        assert_eq!(gradient.stops[0].color, color);
    }
    for html in ["<div id=x></div><div id=x></div>",
        "<div id=x></div><div data-nuxie-id=x></div>",
        "<div id=x style='display:none'><div id=x></div></div>"] {
        assert_eq!(compile(&input(html, "div{background:linear-gradient(red,blue)}"))
            .unwrap_err()[0].code, "duplicate-id");
    }
}

#[test]
fn public_gradient_targets_are_checked_against_imported_object_types() {
    let output = compile(&input("<div id=box></div>",
        "#box{background:linear-gradient(red,blue)}")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let is_layout = |id| artboard.with_artboard(|a| {
        a.objects().get(id as usize).and_then(Option::as_ref).is_some_and(|object|
            object.with(|o| o.as_layout_component().is_some()).unwrap_or(false))
    });
    assert!(output.runtime_requirements.ensure_layout_targets(is_layout).is_ok());
    let style_id = target(&output, "box") + 1;
    assert!(!is_layout(style_id));
    for invalid in [style_id, u32::MAX] {
        let mut requirements = output.runtime_requirements.clone();
        requirements.layout_linear_gradients[0].object_id = invalid;
        assert_eq!(requirements.ensure_layout_targets(is_layout).unwrap_err().code,
            "invalid-layout-linear-gradient-target");
    }
}
