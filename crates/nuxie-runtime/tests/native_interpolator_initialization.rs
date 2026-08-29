//! Native initialization regressions grounded in the pinned interpolator owners:
//! `backboard_importer.cpp`, `cubic_{interpolator,ease_interpolator,value_interpolator}.cpp`,
//! and `elastic_{interpolator,ease}.cpp`. No alternate interpolation implementation.

use nuxie_runtime::source::{
    animation::{
        cubic_ease_interpolator::CubicEaseInterpolator,
        cubic_value_interpolator::CubicValueInterpolator,
        elastic_interpolator::ElasticInterpolator,
    },
    artboard::Artboard,
    backboard::Backboard,
    core::{CoreArena, CoreHandle, binary_reader::BinaryReader},
    generated::{
        animation::{
            cubic_interpolator_base::CubicInterpolatorBase,
            elastic_interpolator_base::ElasticInterpolatorBase,
        },
        backboard_base::BackboardBase,
    },
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
};

fn deserialize_float(owner: &CoreHandle, key: u16, value: f32) {
    let bytes = value.to_le_bytes();
    let mut reader = BinaryReader::new(&bytes);
    assert_eq!(
        owner.with_mut(|owner| owner.deserialize(key, &mut reader)),
        Some(true)
    );
    assert!(!reader.has_error());
    assert!(reader.reached_end());
}

fn set_cubic_controls(owner: &CoreHandle, y1: f32, y2: f32) {
    deserialize_float(owner, CubicInterpolatorBase::X1_PROPERTY_KEY, 0.25);
    deserialize_float(owner, CubicInterpolatorBase::X2_PROPERTY_KEY, 0.25);
    deserialize_float(owner, CubicInterpolatorBase::Y1_PROPERTY_KEY, y1);
    deserialize_float(owner, CubicInterpolatorBase::Y2_PROPERTY_KEY, y2);
}

fn backboard_stack(arena: &CoreArena) -> ImportStack {
    let backboard = arena.insert(Backboard::default());
    let mut stack = ImportStack::default();
    assert_eq!(
        stack.make_latest(
            BackboardBase::TYPE_KEY,
            Some(Box::new(BackboardImporter::new(backboard)))
        ),
        StatusCode::Ok
    );
    stack
}

fn import_global(owner: &CoreHandle, stack: &mut ImportStack) {
    // Exercise the actual virtual import while this same occurrence is borrowed,
    // just as File::read does. Backboard registration must not reborrow its slot.
    assert_eq!(
        owner.with_mut(|owner| owner.import(stack)),
        Some(StatusCode::Ok)
    );
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.00001,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn global_cubic_is_initialized_during_backboard_registration() {
    let arena = CoreArena::default();
    let mut stack = backboard_stack(&arena);
    let cubic = arena.insert(CubicEaseInterpolator::default());
    set_cubic_controls(&cubic, 0.0, 1.0);
    import_global(&cubic, &mut stack);

    // Independent analytic point: at t=1/2, Bezier x=(1/4,1/4) is
    // 5/16 and y=(0,1) is 1/2. This must work immediately after import,
    // before BackboardImporter::resolve or any artboard onAddedDirty call.
    cubic
        .with_mut(|owner| {
            assert_near(owner.keyframe_interpolator_transform(0.3125).unwrap(), 0.5);
            assert_near(
                owner
                    .keyframe_interpolator_transform_value(2.0, 10.0, 0.3125)
                    .unwrap(),
                6.0,
            );
        })
        .unwrap();
    assert_eq!(stack.resolve(), StatusCode::Ok);
}

#[test]
fn cubic_value_dirty_initialization_recomputes_authored_controls_with_equal_endpoints() {
    let arena = CoreArena::default();
    let cubic = arena.insert(CubicValueInterpolator::default());
    set_cubic_controls(&cubic, 2.0, 2.0);
    let mut context = Artboard::default();
    assert_eq!(
        cubic.with_mut(|owner| owner.on_added_dirty(&mut context)),
        Some(StatusCode::Ok)
    );

    // The constructor cached coefficients before deserialization. Both endpoints
    // stay zero, so transformValue cannot be relied on to rebuild them.
    // At t=1/2 the authored y control points (2,2) produce 3/2.
    let value = cubic
        .with_mut(|owner| {
            owner
                .keyframe_interpolator_transform_value(0.0, 0.0, 0.3125)
                .unwrap()
        })
        .unwrap();
    assert_near(value, 1.5);
}

#[test]
fn elastic_global_and_artboard_initialization_use_authored_amplitude_and_period() {
    // These closed-form points follow pinned ElasticEase::easeOut, rather than
    // calling ElasticEase to generate the oracle. Authored period 0 means 1/2.
    for (period, factor, expected) in [
        (1.0, 0.25, 1.0 + 6.0_f32.sqrt() / 8.0),
        (0.0, 0.125, 1.0 + 3.0_f32.sqrt() * 2.0_f32.powf(-1.25)),
    ] {
        let arena = CoreArena::default();
        let mut stack = backboard_stack(&arena);
        let mut context = Artboard::default();
        for global in [true, false] {
            let elastic = arena.insert(ElasticInterpolator::default());
            deserialize_float(
                &elastic,
                ElasticInterpolatorBase::AMPLITUDE_PROPERTY_KEY,
                2.0,
            );
            deserialize_float(
                &elastic,
                ElasticInterpolatorBase::PERIOD_PROPERTY_KEY,
                period,
            );
            if global {
                import_global(&elastic, &mut stack);
            } else {
                assert_eq!(
                    elastic.with_mut(|owner| owner.on_added_dirty(&mut context)),
                    Some(StatusCode::Ok)
                );
            }
            let value = elastic
                .with_mut(|owner| owner.keyframe_interpolator_transform(factor).unwrap())
                .unwrap();
            assert_near(value, expected);
        }
        assert_eq!(stack.resolve(), StatusCode::Ok);
    }
}
