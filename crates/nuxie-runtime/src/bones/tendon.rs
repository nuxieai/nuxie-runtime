//! Direct Rust owner for pinned C++ `include/rive/bones/tendon.hpp` and
//! `src/bones/tendon.cpp`.
//!
//! Artboard retains authored-order dirty/clean lifecycle orchestration and
//! delegates each Tendon occurrence here.

use anyhow::Context;

use crate::components::{ComponentHandle, Mat2D};
use crate::objects::InstanceObjectArena;
use crate::properties::property_key_for_name;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTendonState {
    pub(crate) inverse_bind: Mat2D,
    pub(crate) bone: Option<ComponentHandle>,
}

impl Default for RuntimeTendonState {
    fn default() -> Self {
        Self {
            inverse_bind: Mat2D::IDENTITY,
            bone: None,
        }
    }
}

impl RuntimeTendonState {
    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }
}

fn generated_bind(objects: &InstanceObjectArena, local_id: usize) -> Mat2D {
    let value = |name, default| {
        property_key_for_name("Tendon", name)
            .and_then(|key| objects.double_property(local_id, key))
            .unwrap_or(default)
    };
    Mat2D([
        value("xx", 1.0),
        value("xy", 0.0),
        value("yx", 0.0),
        value("yy", 1.0),
        value("tx", 0.0),
        value("ty", 0.0),
    ])
}

/// Move-only extraction of the established Tendon dirty-phase state retention.
///
/// Pinned C++ computes the inverse bind before resolving the Bone
/// (`src/bones/tendon.cpp:8-39`). The current Rust adapter resolves first; that
/// malformed-import ordering remains an explicit semantic closure item rather
/// than being hidden inside this structural commit.
pub(crate) fn on_added_dirty(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
    local_id: usize,
) -> anyhow::Result<()> {
    let bone_id_key = property_key_for_name("Tendon", "boneId")
        .context("Tendon.boneId is missing from the runtime schema")?;
    let bone_local = objects
        .uint_property(local_id, bone_id_key)
        .and_then(|bone| usize::try_from(bone).ok())
        .context("Tendon boneId does not resolve to an object slot")?;
    let bone = objects
        .component_handle(bone_local)
        .context("Tendon boneId does not resolve to a Component occurrence")?;
    if objects
        .component(bone)
        .and_then(|bone| bone.concrete.bone.as_ref())
        .is_none()
    {
        anyhow::bail!("Tendon boneId does not resolve to a Bone");
    }
    let inverse_bind = generated_bind(objects, local_id).invert_or_identity();
    let tendon = objects
        .component_mut(handle)
        .expect("Tendon handle was validated")
        .concrete
        .tendon
        .as_mut()
        .expect("Tendon occurrence owns Tendon state");
    tendon.inverse_bind = inverse_bind;
    tendon.bone = Some(bone);
    Ok(())
}

/// `Tendon::onAddedClean` registers once on its retained Skin parent
/// (`src/bones/tendon.cpp:41-52`).
pub(crate) fn on_added_clean(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> anyhow::Result<()> {
    let parent = objects
        .component(handle)
        .and_then(|component| component.parent)
        .context("Tendon is missing its parent Component")?;
    let Some(skin) = objects
        .component_mut(parent)
        .and_then(|parent| parent.concrete.skin.as_mut())
    else {
        anyhow::bail!("Tendon parent is not a Skin");
    };
    skin.tendons.push(handle);
    Ok(())
}
