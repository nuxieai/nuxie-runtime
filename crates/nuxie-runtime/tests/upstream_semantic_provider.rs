//! Direct ports of all six cases in pinned
//! `tests/unit_tests/runtime/semantic_provider_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    core::CoreHandle,
    semantic::{
        semantic_data::SemanticData,
        semantic_provider::{
            ResolvedSemanticData, can_infer_semantics, resolve_semantic_data, semantic_bounds,
        },
    },
    text::text::Text,
};
use nuxie_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn simpsons() -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(
        &pinned_fixture("semantic/simpsons.riv"),
        factory,
        None,
        None,
        None,
    )
    .expect("semantic/simpsons.riv imports");
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard");
    let state_machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine 0");
    for _ in 0..10 {
        state_machine.advance_and_apply(0.1);
    }
    (file, artboard, state_machine)
}

fn semantic_data_hosts(artboard: &RuntimeArtboardInstanceHandle) -> Vec<(CoreHandle, CoreHandle)> {
    let semantic_data = artboard.with_artboard(|artboard| {
        artboard
            .objects_typed::<SemanticData>()
            .iter()
            .collect::<Vec<_>>()
    });
    semantic_data
        .into_iter()
        .filter_map(|data| {
            let host = data
                .with(|data| {
                    data.as_component()
                        .and_then(|component| component.parent_handle())
                })
                .flatten()?;
            Some((data, host))
        })
        .collect()
}

#[test]
fn can_infer_semantics_null_returns_false() {
    let (_file, _artboard, _machine) = simpsons();
    assert!(!can_infer_semantics(None));
}

#[test]
fn resolve_semantic_data_null_returns_default() {
    let (_file, _artboard, _machine) = simpsons();
    let resolved = resolve_semantic_data(None);
    assert_eq!(resolved, ResolvedSemanticData::default());
    assert!(!resolved.has_semantics);
    assert_eq!(resolved.role, 0);
    assert!(resolved.label.is_empty());
}

#[test]
fn semantic_bounds_null_returns_empty() {
    let (_file, _artboard, _machine) = simpsons();
    let bounds = semantic_bounds(None);
    assert!(bounds.is_empty_or_nan());
}

#[test]
fn can_infer_semantics_text_returns_true_for_any_text_component() {
    let (_file, artboard, _machine) = simpsons();
    let texts = artboard
        .with_artboard(|artboard| artboard.objects_typed::<Text>().iter().collect::<Vec<_>>());
    let mut saw_text = false;
    for text in texts {
        assert!(can_infer_semantics(Some(&text)));
        saw_text = true;
    }
    assert!(saw_text);
}

#[test]
fn resolve_semantic_data_on_a_node_hosting_a_semantic_data_child_uses_the_explicit_role_and_label()
{
    let (_file, artboard, _machine) = simpsons();
    let mut checked = 0;
    for (data, host) in semantic_data_hosts(&artboard) {
        let resolved = resolve_semantic_data(Some(&host));
        let (role, label) = data
            .with_downcast::<SemanticData, _>(|data| {
                (data.base.role(), data.base.label().to_owned())
            })
            .expect("SemanticData");
        assert!(resolved.has_semantics);
        assert_eq!(resolved.role, role);
        assert_eq!(resolved.label.as_bytes(), label.as_bytes());
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn semantic_bounds_on_a_laid_out_drawable_node_produces_non_empty_bounds() {
    let (_file, artboard, _machine) = simpsons();
    let mut checked = 0;
    for (data, host) in semantic_data_hosts(&artboard) {
        let label = data
            .with_downcast::<SemanticData, _>(|data| data.base.label().to_owned())
            .expect("SemanticData.label");
        let bounds = semantic_bounds(Some(&host));
        assert!(!bounds.is_empty_or_nan(), "{label}");
        assert!(bounds.max_x - bounds.min_x > 0.0, "{label}");
        assert!(bounds.max_y - bounds.min_y > 0.0, "{label}");
        checked += 1;
    }
    assert!(checked > 0);
}
