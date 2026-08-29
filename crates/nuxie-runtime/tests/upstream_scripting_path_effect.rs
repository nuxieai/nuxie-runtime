//! Direct port of pinned
//! `tests/unit_tests/runtime/scripting/scripting_path_effect_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    assets::script_asset::ScriptAsset, lua::scripting_vm::RuntimeScriptingVmHandle,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};

use nuxie_sriv as sriv;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn reusing_a_path_in_multiple_passes_works_correctly() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let vm = RuntimeScriptingVmHandle::new(Box::new(
        ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default()).unwrap(),
    ));
    // Native TextAssetImporter verifies the fixture's production signature.
    // The tools feature does not opt into the separate sample signing key.
    let file = File::import(
        &pinned_fixture("reuse_path_in_effect.riv"),
        RuntimeFactoryHandle::from_factory(&mut silver).unwrap(),
        None,
        None,
        Some(vm),
    )
    .expect("reuse_path_in_effect.riv imports");
    let scripts = file.with_file(|file| {
        file.assets()
            .iter()
            .filter_map(|asset| asset.with_downcast::<ScriptAsset, _>(|script| script.verified()))
            .collect::<Vec<_>>()
    });
    assert!(!scripts.is_empty());
    assert!(
        scripts.into_iter().all(|verified| verified),
        "signed upstream scripts verify"
    );
    let artboard = file.with_file(File::artboard_default).unwrap();
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    let model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
    let mut renderer = silver.borrow().make_renderer();
    machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);
    silver.borrow_mut().add_frame();
    let expected = pinned_fixture("../silvers/reuse_path_in_effect.sriv");
    let actual = silver.borrow().bytes().to_vec();
    assert_eq!(actual.len(), expected.len(), "pinned SRIV byte length");
    sriv::compare_sriv(
        &sriv::parse_sriv(&expected).unwrap(),
        &sriv::parse_sriv(&actual).unwrap(),
    )
    .expect("pinned reuse_path_in_effect silver");
}
