//! Pinned `blend_animation.hpp/.cpp`: animation() returns m_EmptyAnimation when
//! import leaves m_Animation null. This is exercised by rewards_demo's Heart.

use nuxie_runtime::source::{
    animation::{
        blend_animation_1d::BlendAnimation1D, blend_state_1d_input::BlendState1DInput,
        blend_state_instance::BlendStateAnimationInstance, linear_animation::LinearAnimation,
    },
    artboard::{Artboard, RuntimeArtboardInstanceWeakHandle},
    core::{CoreArena, CoreHandle, binary_reader::BinaryReader},
    generated::{
        animation::{blend_animation_base::BlendAnimationBase, layer_state_base::LayerStateBase},
        artboard_base::ArtboardBase,
    },
    importers::{
        artboard_importer::ArtboardImporter, import_stack::ImportStack,
        layer_state_importer::LayerStateImporter,
    },
    status_code::StatusCode,
};

fn import_blend(arena: &CoreArena, stack: &mut ImportStack, id: Option<u8>) -> CoreHandle {
    let blend = arena.insert(BlendAnimation1D::default());
    if let Some(id) = id {
        // All explicit IDs in this test fit the format's one-byte varuint.
        let bytes = [id];
        let mut reader = BinaryReader::new(&bytes);
        assert_eq!(
            blend.with_mut(|owner| {
                owner.deserialize(BlendAnimationBase::ANIMATION_ID_PROPERTY_KEY, &mut reader)
            }),
            Some(true)
        );
        assert!(reader.reached_end());
        assert!(!reader.has_error());
    }
    assert_eq!(
        blend.with_mut(|owner| owner.import(stack)),
        Some(StatusCode::Ok)
    );
    blend
}

fn animation(blend: &CoreHandle) -> CoreHandle {
    blend
        .with(|owner| owner.blend_animation_animation())
        .flatten()
        .unwrap()
}

#[test]
fn invalid_animation_ids_share_the_pinned_default_animation() {
    let arena = CoreArena::default();
    let artboard = arena.insert(Artboard::default());
    let authored = arena.insert(LinearAnimation::default());
    let state = arena.insert(BlendState1DInput::default());
    let mut artboard_importer = ArtboardImporter::new(artboard);
    artboard_importer.add_animation(authored.clone());
    let mut stack = ImportStack::default();
    assert_eq!(
        stack.make_latest(ArtboardBase::TYPE_KEY, Some(Box::new(artboard_importer))),
        StatusCode::Ok
    );
    assert_eq!(
        stack.make_latest(
            LayerStateBase::TYPE_KEY,
            Some(Box::new(LayerStateImporter::new(state)))
        ),
        StatusCode::Ok
    );

    let unset = import_blend(&arena, &mut stack, None);
    let out_of_range = import_blend(&arena, &mut stack, Some(1));
    let valid = import_blend(&arena, &mut stack, Some(0));
    let empty = animation(&unset);
    assert_eq!(empty, animation(&out_of_range));
    assert_ne!(empty, authored);
    assert_eq!(animation(&valid), authored);

    // m_EmptyAnimation has the ordinary, empty C++ constructor. In particular
    // it is a one-second animation, not a fabricated zero-duration sentinel.
    empty
        .with_downcast::<LinearAnimation, _>(|animation| {
            assert_eq!(animation.base.base.name(), "");
            assert_eq!(animation.num_keyed_objects(), 0);
            assert_eq!(animation.base.fps(), 60);
            assert_eq!(animation.base.duration(), 60);
            assert_eq!(animation.base.speed(), 1.0);
            assert_eq!(animation.base.loop_value(), 0);
            assert_eq!(animation.base.work_start(), u32::MAX);
            assert_eq!(animation.base.work_end(), u32::MAX);
            assert!(!animation.base.enable_work_area());
            assert!(!animation.base.quantize());
            assert_eq!(animation.duration_seconds(), 1.0);
        })
        .unwrap();

    // Exercise the exact instance constructor that previously rejected Heart's
    // unassigned blend. No artboard is needed for this empty definition's clock.
    let instance = BlendStateAnimationInstance::<BlendAnimation1D>::new(
        unset,
        RuntimeArtboardInstanceWeakHandle::default(),
    );
    assert_eq!(instance.animation_instance().time(), 0.0);
    assert_eq!(instance.animation_instance().duration_seconds(), 1.0);
}
