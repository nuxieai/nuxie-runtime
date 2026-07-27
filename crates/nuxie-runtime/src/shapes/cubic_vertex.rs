//! Direct Rust owner for pinned C++ `include/rive/shapes/cubic_vertex.hpp`
//! and `src/shapes/cubic_vertex.cpp`.

use crate::ArtboardInstance;
use crate::bones::cubic_weight::RuntimeCubicWeightState;
use crate::bones::weight::deform_point_from_skin;
use crate::components::Mat2D;
use crate::properties::property_key_for_name;

/// Direct `CubicVertex::xChanged` / `CubicVertex::yChanged`: Rust computes
/// control points on demand, so the C++ cache-invalidating tail is structural
/// rather than stored; the inherited Vertex callback still dirties geometry.
pub(crate) fn apply_position_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> bool {
    if !matches!(
        type_name,
        Some("CubicMirroredVertex" | "CubicAsymmetricVertex" | "CubicDetachedVertex")
    ) {
        return false;
    }
    super::vertex::apply_position_property_changed(artboard, local_id, type_name, property_key)
}

pub(super) fn deform_weight(
    artboard: &ArtboardInstance,
    weight_local: usize,
    in_point: (f32, f32),
    out_point: (f32, f32),
    skin_world: Mat2D,
    bone_transforms: &[Mat2D],
) -> Option<RuntimeCubicWeightState> {
    let packed = |field, default| {
        property_key_for_name("CubicWeight", field)
            .and_then(|key| artboard.uint_property(weight_local, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(default)
    };
    let in_translation = deform_point_from_skin(
        in_point,
        packed("inIndices", 1),
        packed("inValues", 255),
        skin_world,
        bone_transforms,
    )?;
    let out_translation = deform_point_from_skin(
        out_point,
        packed("outIndices", 1),
        packed("outValues", 255),
        skin_world,
        bone_transforms,
    )?;
    Some(RuntimeCubicWeightState {
        in_translation,
        out_translation,
    })
}
