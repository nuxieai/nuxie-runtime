//! Both runtime/blend_accumulator_test.cpp cases at upstream 75a22f94.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        blend_accumulator::BlendAccumulator, blend_state_1d::BlendState1D,
        layer_state_flags::LayerStateFlags, state_machine::StateMachine,
        state_machine_layer::StateMachineLayer,
    },
    artboard::Artboard,
    component::ComponentDirt,
    core::CoreType,
    generated::{
        animation::layer_state_base::LayerStateBase, core_registry::CoreRegistry,
        node_base::NodeBase,
    },
    node::Node,
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};

fn file() -> RuntimeFileHandle {
    let upstream = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(upstream).join("tests/unit_tests/assets/blend_test.riv"),
    )
    .expect("pinned blend fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}

#[test]
fn a_blend_accumulator_mixes_like_writes_to_the_object() {
    let file = file();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let nodes = artboard.with_artboard(|a| a.find_all_handles::<Node>());
    assert!(nodes.len() >= 2);
    let seeded = &nodes[0];
    let unseeded = &nodes[1];
    let x = i32::from(NodeBase::X_PROPERTY_KEY);
    CoreRegistry::set_double_handle(unseeded, x, 7.0);
    let mut accumulator = BlendAccumulator::default();
    accumulator.seed_double(seeded, x, 10.0);
    accumulator.apply_double(seeded, x, 0.5, 30.0);
    accumulator.apply_double(seeded, x, 0.25, 50.0);
    accumulator.apply_double(unseeded, x, 0.5, 20.0);
    assert_eq!(CoreRegistry::get_double_handle(unseeded, x), Some(7.0));
    accumulator.flush();
    let mut expected = 10.0;
    expected = expected * 0.5 + 30.0 * 0.5;
    expected = expected * 0.75 + 50.0 * 0.25;
    assert_eq!(CoreRegistry::get_double_handle(seeded, x), Some(expected));
    assert_eq!(
        CoreRegistry::get_double_handle(unseeded, x),
        Some(7.0 * 0.5 + 20.0 * 0.5)
    );
    accumulator.apply_double(unseeded, x, 0.5, 0.0);
    accumulator.flush();
    assert_eq!(
        CoreRegistry::get_double_handle(unseeded, x),
        Some((7.0 * 0.5 + 20.0 * 0.5) * 0.5)
    );
}

#[test]
fn a_settled_blend_state_does_not_dirty_what_it_blends() {
    let file = file();
    let source = file.with_file(File::artboard).unwrap();
    let definition = source
        .with_downcast::<Artboard, _>(|a| a.state_machine_named("blend"))
        .flatten()
        .unwrap();
    let layer = definition
        .with_downcast::<StateMachine, _>(|m| m.layer(0))
        .flatten()
        .unwrap();
    let states = layer
        .with_downcast::<StateMachineLayer, _>(|l| l.states().to_vec())
        .unwrap();
    let mut blend_states = 0;
    for state in states {
        if state.is_type_of(BlendState1D::TYPE_KEY) {
            let key = i32::from(LayerStateBase::FLAGS_PROPERTY_KEY);
            let flags = CoreRegistry::get_uint_handle(&state, key).unwrap();
            CoreRegistry::set_uint_handle(&state, key, flags | u32::from(LayerStateFlags::RESET.0));
            blend_states += 1;
        }
    }
    assert!(blend_states > 0);
    let artboard = file.with_file(File::artboard_default).unwrap();
    let machine = artboard.state_machine_named("blend").unwrap();
    for _ in 0..60 {
        machine.advance_and_apply(1.0 / 60.0);
    }
    let has_dirt = || {
        artboard
            .core_handle()
            .with(|a| {
                a.as_component()
                    .unwrap()
                    .has_dirt(ComponentDirt::COMPONENTS)
            })
            .unwrap()
    };
    assert!(!has_dirt());
    machine.with_instance_mut(|m| m.advance_seconds(0.0));
    assert!(!has_dirt());
}
