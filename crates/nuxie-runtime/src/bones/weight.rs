use anyhow::{Context, Result};

use crate::components::ComponentHandle;
use crate::components::Mat2D;
use crate::objects::InstanceObjectArena;

use super::cubic_weight::RuntimeCubicWeightState;

/// Runtime-only fields owned by C++ `Weight`.
///
/// Packed values and indices remain in the occurrence's generated storage;
/// only the settled deformation output lives on the concrete owner
/// (`include/rive/bones/weight.hpp:12-15`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct RuntimeWeightState {
    pub(crate) translation: (f32, f32),
    pub(crate) cubic: Option<RuntimeCubicWeightState>,
}

impl RuntimeWeightState {
    pub(crate) fn new(is_cubic: bool) -> Self {
        Self {
            cubic: is_cubic.then(RuntimeCubicWeightState::default),
            ..Self::default()
        }
    }
}

/// Pinned `Weight::onAddedDirty`: retain the exact parent Vertex occurrence.
pub(crate) fn on_added_dirty(
    objects: &mut InstanceObjectArena,
    weight: ComponentHandle,
) -> Result<()> {
    let parent = objects
        .component(weight)
        .and_then(|component| component.parent)
        .context("Weight is missing its parent Component")?;
    if objects
        .component(parent)
        .and_then(|parent| parent.concrete.vertex.as_ref())
        .is_none()
    {
        anyhow::bail!("Weight parent is not a Vertex");
    }
    objects
        .component_mut(parent)
        .expect("Vertex parent handle was validated")
        .concrete
        .vertex
        .as_mut()
        .expect("Weight parent owns Vertex state")
        .weight = Some(weight);
    Ok(())
}

pub(crate) fn deform_point_from_skin(
    point: (f32, f32),
    indices: u32,
    weights: u32,
    skin_world: Mat2D,
    bone_transforms: &[Mat2D],
) -> Option<(f32, f32)> {
    let mut blended = [0.0; 6];
    for index in 0..4 {
        let shift = index * 8;
        let weight = ((weights >> shift) & 0xff) as u8;
        if weight == 0 {
            continue;
        }
        let bone_index = ((indices >> shift) & 0xff) as usize;
        let bone_transform = bone_transforms.get(bone_index)?;
        let normalized_weight = f32::from(weight) / 255.0;
        for (target, value) in blended.iter_mut().zip(bone_transform.0) {
            // Clang contracts each C++ `accumulator += value * weight` in
            // `Weight::deform`; Rust does not contract implicitly. Preserve
            // that per-site rounding exactly (`src/bones/weight.cpp:35-48`;
            // `docs/PORTING.md` §4.2).
            *target = value.mul_add(normalized_weight, *target);
        }
    }
    let skinned = skin_world.transform_point(point.0, point.1);
    Some(Mat2D(blended).transform_point(skinned.0, skinned.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deformation_matches_clang_contracted_accumulation() {
        // The two active packed influences are the minimal tape.riv
        // counterexample for the one-ulp difference between separate
        // multiply/add and the C++ geometry pipeline's default contraction.
        // Preserve Weight::deform's source loop and clang rounding exactly
        // (`src/bones/weight.cpp:24-55`; `docs/PORTING.md` §4.2).
        let f = f32::from_bits;
        let bones = [
            Mat2D::IDENTITY,
            Mat2D([
                f(1_057_841_340),
                f(3_194_226_667),
                f(1_049_771_165),
                f(1_057_885_778),
                f(1_107_583_000),
                f(1_124_670_380),
            ]),
            Mat2D([
                f(1_057_841_342),
                f(3_194_226_637),
                f(1_049_771_148),
                f(1_057_885_779),
                f(1_106_043_744),
                f(1_125_092_309),
            ]),
        ];
        let point = (f(3_265_177_136), f(1_091_681_984));
        let world = Mat2D([1.0, 0.0, 0.0, 1.0, f(1_135_168_757), f(1_135_822_912)]);

        let deformed =
            deform_point_from_skin(point, 258, 32_385, world, &bones).expect("valid bone indices");
        assert_eq!(
            (deformed.0.to_bits(), deformed.1.to_bits()),
            (1_133_233_234, 1_133_466_484)
        );
    }
}
