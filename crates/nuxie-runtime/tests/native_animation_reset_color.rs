//! Pinned AnimationResetFactory stores CoreRegistry's signed color int as float;
//! AnimationReset::apply converts that float back to the signed color argument.
//! The transition_actions fixture exposed unsigned conversion losing low bits.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    source::{
        animation::animation_reset_factory::AnimationResetFactory,
        generated::{core_registry::CoreRegistry, shapes::paint::solid_color_base::SolidColorBase},
    },
};
use std::path::PathBuf;

#[test]
fn animation_reset_preserves_signed_color_bits_from_live_and_baseline_values() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes =
        std::fs::read(PathBuf::from(root).join("tests/unit_tests/assets/transition_actions.riv"))
            .expect("pinned transition-actions fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&bytes, retained, None, None, None).unwrap();
    let mut artboard = file.with_file(File::artboard_default).unwrap();
    let (animation, color) = artboard.with_artboard(|artboard| {
        (
            artboard.animation_handle_at(1).unwrap(),
            artboard.resolve_handle(9).unwrap(),
        )
    });
    assert!(color.is_type_of(SolidColorBase::TYPE_KEY));
    let key = i32::from(SolidColorBase::COLOR_VALUE_PROPERTY_KEY);

    // This is the fixture's Timeline 2 keyframe color, also emitted by the
    // pinned silver. Its signed value is exactly representable in f32, whereas
    // converting its unsigned magnitude rounds it down by 23.
    let expected = 4_278_237_207_u32 as i32;
    for baseline in [false, true] {
        CoreRegistry::set_color_handle(&color, key, expected);
        let reset = artboard.with_artboard(|artboard| {
            AnimationResetFactory::from_animation_handles(
                std::slice::from_ref(&animation),
                artboard,
                baseline,
            )
        });
        assert!(CoreRegistry::set_color_handle(&color, key, 0));
        reset.apply(&mut artboard);
        assert_eq!(
            CoreRegistry::get_color_handle(&color, key),
            Some(expected),
            "baseline={baseline}: reset restores the exact signed packed color"
        );
        AnimationResetFactory::release(reset);
    }
}
