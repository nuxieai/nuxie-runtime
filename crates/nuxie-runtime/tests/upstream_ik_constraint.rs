//! Direct translation of the six-draw silver test in
//! `tests/unit_tests/runtime/ik_constraint_test.cpp` at upstream
//! d25e6a4b6c1b8382b588f08371231373780fbcd5.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
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
fn ik_constraint_with_non_full_strength() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture("ik_anim_test.riv"),
        RuntimeFactoryHandle::from_factory(&mut silver).unwrap(),
        None,
        None,
        None,
    )
    .expect("ik_anim_test.riv imports");
    let artboard = file.with_file(File::artboard_default).unwrap();
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut renderer = silver.borrow().make_renderer();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    let model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let model = file.with_file(|file| file.create_view_model_instance_at(model_id as usize, 0));
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    artboard.draw(&mut renderer);

    silver.borrow_mut().add_frame();
    machine.advance_and_apply(0.1f32);
    artboard.draw(&mut renderer);

    let frames = (2.0f32 / 0.5f32) as i32;
    for _ in 0..frames {
        silver.borrow_mut().add_frame();
        machine.advance_and_apply(0.5f32);
        artboard.draw(&mut renderer);
    }

    let expected = pinned_fixture("../silvers/ik_anim_test.sriv");
    let actual = silver.borrow().bytes().to_vec();
    assert_eq!(actual.len(), expected.len(), "pinned SRIV byte length");
    sriv::compare_sriv(
        &sriv::parse_sriv(&expected).unwrap(),
        &sriv::parse_sriv(&actual).unwrap(),
    )
    .expect("pinned ik_anim_test silver");
}
