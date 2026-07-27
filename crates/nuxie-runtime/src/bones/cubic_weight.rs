use crate::artboard::ArtboardInstance;
use crate::components::Mat2D;
use crate::properties::property_key_for_name;

use super::weight::deform_point_from_skin;

/// Runtime-only fields owned by C++ `CubicWeight`.
///
/// The inherited base translation remains on `RuntimeWeightState`; this
/// derived owner retains only its independent in/out translations, matching
/// `include/rive/bones/cubic_weight.hpp:9-15`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct RuntimeCubicWeightState {
    pub(crate) in_translation: (f32, f32),
    pub(crate) out_translation: (f32, f32),
}

pub(super) fn deform_runtime_cubic_weight(
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
