//! Direct owner for pinned `src/constraints/transform_constraint.cpp`.

use super::*;

/// Runtime-only fields owned by one pinned C++ `TransformConstraint`.
///
/// `TransformConstraint::constrainWorld` receives both fields by value, then
/// replaces only those local copies with decomposed transforms. Consequently
/// the retained header fields remain default-initialized for the lifetime of
/// the object.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct RuntimeTransformConstraintState {
    pub(crate) components_a: TransformComponents,
    pub(crate) components_b: TransformComponents,
}

/// Direct routing for `TransformConstraint::{originX,originY}Changed()`.
pub(super) fn double_property_changed(property_key: u16) -> bool {
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    property_key == keys.origin_x || property_key == keys.origin_y
}

pub(super) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
    state: RuntimeConstraintState,
) -> bool {
    // Ported from C++ `src/constraints/transform_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    let Some(target_index) = state.target else {
        return false;
    };
    if artboard.component_at(target_index).is_collapsed() {
        return false;
    }

    let transform_a = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    let mut transform_b = target_transform(
        artboard,
        target_index,
        retained_constraint_double(artboard, constraint_local, keys.origin_x, 0.0),
        retained_constraint_double(artboard, constraint_local, keys.origin_y, 0.0),
    );
    if retained_constraint_space(artboard, constraint_local, keys.source_space)
        == TransformSpace::Local
    {
        let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
            return false;
        };
        transform_b = inverse.multiply(transform_b);
    }
    if retained_constraint_space(artboard, constraint_local, keys.dest_space)
        == TransformSpace::Local
    {
        transform_b = parent_world_transform(artboard, component_index).multiply(transform_b);
    }

    let (components_a, components_b) = match state.scratch {
        RuntimeConstraintScratch::Transform(scratch) => {
            (scratch.components_a, scratch.components_b)
        }
        _ => (TransformComponents::default(), TransformComponents::default()),
    };
    let constrained = constrain_world(
        transform_a,
        components_a,
        transform_b,
        components_b,
        retained_constraint_double(artboard, constraint_local, keys.strength, 1.0),
    );
    write_world_transform(artboard, component_index, constrained)
}

/// Direct port of `TransformConstraint::targetTransform()`.
fn target_transform(
    artboard: &ArtboardInstance,
    target_index: ComponentHandle,
    origin_x: f32,
    origin_y: f32,
) -> Mat2D {
    let (left, top, width, height) = constraint_bounds(artboard, target_index);
    let component = artboard.component_at(target_index);
    component.transform.world_transform.multiply(Mat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        left + width * origin_x,
        top + height * origin_y,
    ]))
}

/// Direct port of `TransformConstraint::constrainWorld()`.
///
/// The scratch parameters deliberately remain by-value, matching the pinned
/// C++ signature rather than mutating the private header fields.
pub(super) fn constrain_world(
    from: Mat2D,
    _components_from: TransformComponents,
    to: Mat2D,
    _components_to: TransformComponents,
    strength: f32,
) -> Mat2D {
    let components_from = from.decompose();
    let mut components_to = to.decompose();

    let two_pi = std::f32::consts::PI * 2.0;
    let angle_a = components_from.rotation % two_pi;
    let angle_b = components_to.rotation % two_pi;
    let mut diff = angle_b - angle_a;
    if diff > std::f32::consts::PI {
        diff -= two_pi;
    } else if diff < -std::f32::consts::PI {
        diff += two_pi;
    }

    let t = strength;
    let ti = 1.0 - t;
    components_to.rotation = angle_a + diff * t;
    components_to.x = components_from.x * ti + components_to.x * t;
    components_to.y = components_from.y * ti + components_to.y * t;
    components_to.scale_x = components_from.scale_x * ti + components_to.scale_x * t;
    components_to.scale_y = components_from.scale_y * ti + components_to.scale_y * t;
    components_to.skew = components_from.skew * ti + components_to.skew * t;

    Mat2D::compose(components_to)
}
