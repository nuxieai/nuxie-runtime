//! Direct ports of all eight cases in
//! `tests/unit_tests/runtime/color_channels_test.cpp` at upstream
//! 74c0d601c516f86db4847521198dba42080db06a.

use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    core::field_types::core_uint_type::CoreUintType,
    generated::{
        core_registry::CoreRegistry, shapes::paint::color_channels_base::ColorChannelsBase,
    },
    node::Node,
    shapes::paint::{gradient_stop::GradientStop, solid_color::SolidColor},
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_sriv as sriv;

#[test]
fn color_channels_read_the_right_byte_of_color_value() {
    let mut solid = SolidColor::default();
    solid.set_color_value(0xAABBCCDDu32 as i32);
    assert_eq!(solid.color_alpha(), 0xAA);
    assert_eq!(solid.color_red(), 0xBB);
    assert_eq!(solid.color_green(), 0xCC);
    assert_eq!(solid.color_blue(), 0xDD);

    let mut stop = GradientStop::default();
    stop.set_color_value(0x11223344u32 as i32);
    assert_eq!(stop.color_alpha(), 0x11);
    assert_eq!(stop.color_red(), 0x22);
    assert_eq!(stop.color_green(), 0x33);
    assert_eq!(stop.color_blue(), 0x44);
}

#[test]
fn setting_a_channel_writes_only_its_byte_and_recomposes_color_value() {
    let mut solid = SolidColor::default();
    solid.set_color_value(0xAABBCCDDu32 as i32);

    solid.set_color_green(0x11);
    assert_eq!(solid.color_value() as u32, 0xAABB11DD);
    assert_eq!(solid.color_alpha(), 0xAA);
    assert_eq!(solid.color_red(), 0xBB);
    assert_eq!(solid.color_blue(), 0xDD);

    solid.set_color_alpha(0x00);
    assert_eq!(solid.color_value() as u32, 0x00BB11DD);
}

#[test]
fn channels_clamp_to_255_instead_of_wrapping() {
    let mut solid = SolidColor::default();
    solid.set_color_value(0x00000000);

    solid.set_color_red(300);
    assert_eq!(solid.color_red(), 0xFF);
    assert_eq!(solid.color_value() as u32, 0x00FF0000);

    CoreRegistry::set_uint(
        &mut solid,
        i32::from(ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY),
        1000,
    );
    assert_eq!(
        CoreRegistry::get_uint(
            &mut solid,
            i32::from(ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY),
        ),
        0xFF,
    );
    assert_eq!(solid.color_value() as u32, 0xFFFF0000);
}

#[test]
fn color_channels_base_from_resolves_consumers_null_otherwise() {
    let mut solid = SolidColor::default();
    let stop = GradientStop::default();
    let node = Node::default();

    assert!(ColorChannelsBase::from(&solid).is_some());
    assert!(ColorChannelsBase::from(&stop).is_some());
    assert!(ColorChannelsBase::from(&node).is_none());

    solid.set_color_value(0xFF000000u32 as i32);
    ColorChannelsBase::from_mut(&mut solid)
        .expect("SolidColor includes ColorChannels")
        .set_color_red(0x80);
    assert_eq!(solid.color_value() as u32, 0xFF800000);
}

#[test]
fn shared_channel_keys_dispatch_through_core_registry_for_both_types() {
    let mut solid = SolidColor::default();
    let mut stop = GradientStop::default();

    CoreRegistry::set_uint(
        &mut solid,
        i32::from(ColorChannelsBase::COLOR_RED_PROPERTY_KEY),
        0x34,
    );
    CoreRegistry::set_uint(
        &mut stop,
        i32::from(ColorChannelsBase::COLOR_RED_PROPERTY_KEY),
        0x34,
    );
    assert_eq!(
        CoreRegistry::get_uint(
            &mut solid,
            i32::from(ColorChannelsBase::COLOR_RED_PROPERTY_KEY),
        ),
        0x34,
    );
    assert_eq!(
        CoreRegistry::get_uint(
            &mut stop,
            i32::from(ColorChannelsBase::COLOR_RED_PROPERTY_KEY),
        ),
        0x34,
    );
    assert_eq!(solid.color_red(), 0x34);
    assert_eq!(stop.color_red(), 0x34);

    solid.set_color_value(0x00000000);
    CoreRegistry::set_uint(
        &mut solid,
        i32::from(ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY),
        0xCD,
    );
    assert_eq!(solid.color_value() as u32, 0xCD000000);
    assert_eq!(
        CoreRegistry::get_uint(
            &mut solid,
            i32::from(ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY),
        ),
        0xCD,
    );
}

#[test]
fn object_supports_property_is_true_for_channels_on_consumers_only() {
    let solid = SolidColor::default();
    let stop = GradientStop::default();
    let node = Node::default();

    assert!(CoreRegistry::object_supports_property(
        &solid,
        u32::from(ColorChannelsBase::COLOR_RED_PROPERTY_KEY),
    ));
    assert!(CoreRegistry::object_supports_property(
        &stop,
        u32::from(ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY),
    ));
    assert!(!CoreRegistry::object_supports_property(
        &node,
        u32::from(ColorChannelsBase::COLOR_RED_PROPERTY_KEY),
    ));
}

#[test]
fn channel_keys_report_a_uint_field_type_for_data_binding() {
    assert_eq!(
        CoreRegistry::property_field_id(i32::from(ColorChannelsBase::COLOR_RED_PROPERTY_KEY)),
        CoreUintType::ID,
    );
    assert_eq!(
        CoreRegistry::property_field_id(i32::from(ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY)),
        CoreUintType::ID,
    );
}

fn pinned_fixture(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "read pinned fixture {}: {error}; run tools/fetch-test-assets.sh",
            path.display(),
        )
    })
}

#[test]
fn silver_test_of_passthrough_properties() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
    let file = File::import(
        &pinned_fixture("color_passthrough_test.riv"),
        retained,
        None,
        None,
        None,
    )
    .expect("color_passthrough_test.riv imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine 0");
    let view_model = file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    });
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
    machine.advance_and_apply(0.1f32);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);

    let frames = (3.0f32 / 0.25f32) as i32;
    for _ in 0..frames {
        silver.borrow_mut().add_frame();
        machine.advance_and_apply(0.25f32);
        artboard.draw(&mut renderer);
    }

    let expected = pinned_fixture("color_passthrough_test.sriv");
    let actual = silver.borrow().bytes().to_vec();
    // Upstream matches checks byte count before its typed epsilon comparison.
    assert_eq!(actual.len(), expected.len(), "pinned SRIV byte length");
    let expected = sriv::parse_sriv(&expected).expect("valid pinned SRIV");
    let actual = sriv::parse_sriv(&actual).expect("valid native SRIV");
    sriv::compare_sriv(&expected, &actual).expect("pinned color passthrough silver");
}
