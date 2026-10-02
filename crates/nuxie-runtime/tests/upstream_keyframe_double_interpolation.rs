//! Regressions for the production C++ linear KeyFrameDouble interpolation.

use nuxie_runtime::source::{
    animation::keyframe_double::KeyFrameDouble,
    core::CoreArena,
    custom_property_number::CustomPropertyNumber,
    generated::{
        animation::{keyframe_base::KeyFrameBase, keyframe_double_base::KeyFrameDoubleBase},
        core_registry::CoreRegistry,
        custom_property_number_base::CustomPropertyNumberBase,
    },
};

fn apply_linear_pair(from_value: f32, to_value: f32, current_time: f32) -> f32 {
    let arena = CoreArena::default();
    let make_keyframe = |frame, value| {
        let handle = arena.insert(KeyFrameDouble::default());
        assert!(CoreRegistry::set_uint_handle(
            &handle,
            i32::from(KeyFrameBase::FRAME_PROPERTY_KEY),
            frame,
        ));
        assert!(CoreRegistry::set_double_handle(
            &handle,
            i32::from(KeyFrameDoubleBase::VALUE_PROPERTY_KEY),
            value,
        ));
        handle
            .with_mut(|object| object.as_key_frame_mut().unwrap().compute_seconds(60))
            .unwrap();
        handle
    };
    let from = make_keyframe(0, from_value);
    let to = make_keyframe(60, to_value);
    let target = arena.insert(CustomPropertyNumber::default());
    let key = i32::from(CustomPropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
    from.with_downcast::<KeyFrameDouble, _>(|from| {
        to.with_downcast::<KeyFrameDouble, _>(|to| {
            assert!(from.apply_interpolation(&target, key, current_time, to, 1.0, None));
        })
        .unwrap();
    })
    .unwrap();
    CoreRegistry::get_double_handle(&target, key).unwrap()
}

#[test]
#[cfg(not(feature = "strict-fp"))]
fn linear_keyframe_preserves_production_contracted_rounding() {
    // Captured from virtualize_blendmode's 82-frame replay at 1.35 seconds.
    // C++ rounds (to-from), then uses fmadd; separate mul/add loses one ULP,
    // changing the data-bound blend weight and ultimately a paint color byte.
    let actual = apply_linear_pair(100.0, 0.0, f32::from_bits(0x3f333334));
    assert_eq!(actual.to_bits(), 0x41effffe);
    assert_eq!((actual / 100.0).to_bits(), 0x3e999998);
}

#[test]
#[cfg(feature = "strict-fp")]
fn linear_keyframe_preserves_strict_source_rounding() {
    let factor = f32::from_bits(0x3f333334);
    let actual = apply_linear_pair(100.0, 0.0, factor);
    let expected = 100.0_f32 + (0.0_f32 - 100.0_f32) * factor;
    assert_eq!(actual.to_bits(), expected.to_bits());
    assert_eq!((actual / 100.0).to_bits(), (expected / 100.0).to_bits());
}

#[test]
fn linear_keyframe_retains_endpoints_and_extrapolation() {
    for (time, expected) in [(0.0, 100.0_f32), (1.0, 0.0), (-0.5, 150.0), (1.5, -50.0)] {
        assert_eq!(
            apply_linear_pair(100.0, 0.0, time).to_bits(),
            expected.to_bits(),
            "time {time}"
        );
    }
}
