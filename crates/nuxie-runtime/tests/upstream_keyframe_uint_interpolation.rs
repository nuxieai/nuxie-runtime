//! Direct port of
//! `tests/unit_tests/runtime/keyframe_uint_interpolation_test.cpp` at upstream
//! 6d6ab6f8102ffdd200f0c8147d339688a91fe867.

use nuxie_runtime::source::{
    animation::keyframe_uint::KeyFrameUint,
    core::CoreArena,
    generated::{
        animation::{keyframe_base::KeyFrameBase, keyframe_uint_base::KeyFrameUintBase},
        core_registry::{CoreRegistry, CoreRegistryObject},
        shapes::paint::color_channels_base::ColorChannelsBase,
        text::text_base::TextBase,
    },
    shapes::paint::solid_color::SolidColor,
    text::text::Text,
};

// Uint keyframes hold by default -- most uint properties are enums, ids, or
// mode flags. The R/G/B/A color channels opt in to interpolation via
// "interpolates": true in their def, which generates
// CoreRegistry::is_interpolatable_uint. These mirror
// packages/rive_core/test/keyframe_uint_interpolation_test.dart; the two
// implementations must agree on rounding or editor preview and runtime
// playback drift.

// Builds a pair of keyframes on the same property and applies the tween at
// `current_time`, returning what landed on the object.
fn apply_pair(
    object: &mut dyn CoreRegistryObject,
    property_key: u16,
    from_value: u32,
    to_value: u32,
    current_time: f32,
) -> u32 {
    let arena = CoreArena::default();
    let from = arena.insert(KeyFrameUint::default());
    assert!(CoreRegistry::set_uint_handle(
        &from,
        i32::from(KeyFrameBase::FRAME_PROPERTY_KEY),
        0,
    ));
    assert!(CoreRegistry::set_uint_handle(
        &from,
        i32::from(KeyFrameUintBase::VALUE_PROPERTY_KEY),
        from_value,
    ));
    from.with_mut(|from| from.as_key_frame_mut().unwrap().compute_seconds(60))
        .unwrap();

    let to = arena.insert(KeyFrameUint::default());
    assert!(CoreRegistry::set_uint_handle(
        &to,
        i32::from(KeyFrameBase::FRAME_PROPERTY_KEY),
        60,
    ));
    assert!(CoreRegistry::set_uint_handle(
        &to,
        i32::from(KeyFrameUintBase::VALUE_PROPERTY_KEY),
        to_value,
    ));
    to.with_mut(|to| to.as_key_frame_mut().unwrap().compute_seconds(60))
        .unwrap();

    from.with_downcast::<KeyFrameUint, _>(|from| {
        to.with(|to| {
            from.apply_interpolation(
                object,
                i32::from(property_key),
                current_time,
                to.as_key_frame().unwrap(),
                1.0,
                None,
            );
        })
        .unwrap();
    })
    .unwrap();
    CoreRegistry::get_uint(object, i32::from(property_key))
}

#[test]
fn interpolatable_uint_whitelist_covers_color_channels() {
    assert!(CoreRegistry::is_interpolatable_uint(u32::from(
        ColorChannelsBase::COLOR_RED_PROPERTY_KEY,
    )));
    assert!(CoreRegistry::is_interpolatable_uint(u32::from(
        ColorChannelsBase::COLOR_GREEN_PROPERTY_KEY,
    )));
    assert!(CoreRegistry::is_interpolatable_uint(u32::from(
        ColorChannelsBase::COLOR_BLUE_PROPERTY_KEY,
    )));
    assert!(CoreRegistry::is_interpolatable_uint(u32::from(
        ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY,
    )));

    // A uint that animates but must keep holding: an enum in disguise.
    assert!(!CoreRegistry::is_interpolatable_uint(u32::from(
        TextBase::VERTICAL_TRIM_TOP_VALUE_PROPERTY_KEY,
    )));
}

#[test]
fn color_channel_keyframes_interpolate() {
    let mut solid = SolidColor::default();
    solid.set_color_value(0xFF000000u32 as i32);

    assert_eq!(
        apply_pair(
            &mut solid,
            ColorChannelsBase::COLOR_RED_PROPERTY_KEY,
            0,
            100,
            0.5,
        ),
        50,
    );

    // Only the keyed channel moved.
    assert_eq!(solid.color_green(), 0);
    assert_eq!(solid.color_blue(), 0);
    assert_eq!(solid.color_alpha(), 0xFF);
}

#[test]
fn interpolated_channel_values_round_to_nearest_byte() {
    let mut solid = SolidColor::default();
    solid.set_color_value(0xFF000000u32 as i32);

    // 3 * 0.5 = 1.5 rounds to 2, matching the Dart side.
    assert_eq!(
        apply_pair(
            &mut solid,
            ColorChannelsBase::COLOR_RED_PROPERTY_KEY,
            0,
            3,
            0.5,
        ),
        2,
    );
}

#[test]
fn non_whitelisted_uints_still_hold() {
    // A bitmask passthrough with no direct accessor at runtime; drive it
    // through the registry the way a keyframe would.
    let mut text = Text::default();
    CoreRegistry::set_uint(
        &mut text,
        i32::from(TextBase::VERTICAL_TRIM_TOP_VALUE_PROPERTY_KEY),
        1,
    );
    assert_eq!(
        CoreRegistry::get_uint(
            &mut text,
            i32::from(TextBase::VERTICAL_TRIM_TOP_VALUE_PROPERTY_KEY),
        ),
        1,
        "the hold assertion must not pass merely because this registry field is a no-op",
    );
    CoreRegistry::set_uint(
        &mut text,
        i32::from(TextBase::VERTICAL_TRIM_TOP_VALUE_PROPERTY_KEY),
        0,
    );

    assert_eq!(
        apply_pair(
            &mut text,
            TextBase::VERTICAL_TRIM_TOP_VALUE_PROPERTY_KEY,
            0,
            2,
            0.5,
        ),
        0,
    );
}
