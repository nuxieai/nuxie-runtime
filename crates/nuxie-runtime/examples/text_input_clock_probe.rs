//! Test-only Wasm entry point: pointer-down on the pinned text-input file.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{math::vec2d::Vec2D, pointer_button::PointerButton};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[cfg_attr(
    all(target_arch = "wasm32", feature = "js-host-seed"),
    wasm_bindgen::prelude::wasm_bindgen
)]
pub fn clock_probe_pointer_down_bytes(bytes: &[u8]) -> u32 {
    #[cfg(all(
        target_arch = "wasm32",
        target_os = "unknown",
        not(feature = "js-host-seed")
    ))]
    nuxie_runtime::source::set_engine_clock(|| 1_000_000);
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("factory");
    let file = File::import(bytes, retained, None, None, None).expect("fixture import");
    let artboard = file
        .with_file(|file| file.artboard_named("SingleLine"))
        .expect("SingleLine");
    let machine = artboard.state_machine_instance_handle(0).expect("machine");
    machine.advance_and_apply(0.0);
    machine.with_instance_mut(|m| {
        m.pointer_down(Vec2D::new(70.0, 160.0), 0, PointerButton::Primary);
        m.pointer_up(Vec2D::new(70.0, 160.0), 0, PointerButton::Primary);
    });
    let manager = machine
        .with_instance(|m| m.focus_manager())
        .expect("focus manager");
    assert!(
        manager.with_focus_manager(|m| m.primary_focus_accepts_text()),
        "pointer-down must focus the native input"
    );
    1
}
