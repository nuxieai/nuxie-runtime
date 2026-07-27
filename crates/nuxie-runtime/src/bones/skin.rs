//! Direct Rust owner for pinned C++ `include/rive/bones/skin.hpp` and
//! `src/bones/skin.cpp`.
//!
//! Tendon registration, Skinnable behavior, and Vertex/Weight deformation
//! stay on their corresponding owners. Artboard retains authored-order
//! construction and dependency scheduling, calling the focused Skin
//! operations here at the same points as before this structural extraction.

use anyhow::Context;

use crate::artboard::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle, Mat2D, RuntimeSkinnableKind};
use crate::objects::InstanceObjectArena;
use crate::properties::property_key_for_name;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSkinState {
    pub(crate) world_transform: Mat2D,
    pub(crate) tendons: Vec<ComponentHandle>,
    pub(crate) skinnable: Option<ComponentHandle>,
    pub(crate) bone_transforms: Vec<Mat2D>,
    #[cfg(test)]
    pub(crate) buffer_rebuilds: usize,
}

impl Default for RuntimeSkinState {
    fn default() -> Self {
        Self {
            world_transform: Mat2D::IDENTITY,
            tendons: Vec::new(),
            skinnable: None,
            bone_transforms: Vec::new(),
            #[cfg(test)]
            buffer_rebuilds: 0,
        }
    }
}

impl RuntimeSkinState {
    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }
}

fn generated_world_transform(objects: &InstanceObjectArena, local_id: usize) -> Mat2D {
    let value = |name, default| {
        property_key_for_name("Skin", name)
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

/// Concrete `Skin::onAddedDirty`: retain the authored skin matrix, resolve
/// the parent Skinnable when present, and link both owners.
pub(crate) fn on_added_dirty(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
    local_id: usize,
) -> anyhow::Result<()> {
    let parent = objects
        .component(handle)
        .and_then(|component| component.parent)
        .context("Skin is missing its parent Component")?;
    let parent_is_skinnable = objects
        .component(parent)
        .and_then(|parent| parent.concrete.skinnable.as_ref())
        .is_some();
    let world_transform = generated_world_transform(objects, local_id);
    objects
        .component_mut(handle)
        .expect("Skin handle was validated")
        .concrete
        .skin
        .as_mut()
        .expect("Skin occurrence owns Skin state")
        .world_transform = world_transform;
    if parent_is_skinnable {
        objects
            .component_mut(handle)
            .expect("Skin handle was validated")
            .concrete
            .skin
            .as_mut()
            .expect("Skin occurrence owns Skin state")
            .skinnable = Some(parent);
        objects
            .component_mut(parent)
            .expect("Skinnable parent handle was validated")
            .concrete
            .skinnable
            .as_mut()
            .expect("Skin parent owns Skinnable state")
            .skin = Some(handle);
    }
    // C++ returns MissingObject for a non-Skinnable parent after retaining
    // m_WorldTransform. Artboard::canContinue makes that status non-fatal, so
    // the established Rust construction keeps the Skin with a null Skinnable.
    Ok(())
}

/// Concrete dependency walk from `Skin::buildDependencies`. Buffer allocation
/// remains a separate call because Artboard merges the retained edge schedule
/// before initializing the owner, preserving the pre-extraction behavior.
pub(crate) fn build_dependencies(objects: &mut InstanceObjectArena, skin: ComponentHandle) {
    let tendons = objects
        .component(skin)
        .and_then(|component| component.concrete.skin.as_ref())
        .map(|skin| skin.tendons.clone())
        .unwrap_or_default();
    for tendon in tendons {
        let Some(bone) = objects
            .component(tendon)
            .and_then(|component| component.concrete.tendon.as_ref())
            .and_then(|tendon| tendon.bone)
        else {
            continue;
        };
        objects.add_dependent(bone, skin);
        let peer_constraints = objects
            .component(bone)
            .and_then(|component| component.concrete.bone.as_ref())
            .map(|bone| bone.peer_constraints.clone())
            .unwrap_or_default();
        for constraint in peer_constraints {
            if let Some(parent) = objects
                .component(constraint)
                .and_then(|component| component.parent)
            {
                objects.add_dependent(parent, skin);
            }
        }
    }
}

pub(crate) fn initialize_bone_transforms(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> anyhow::Result<()> {
    let skin = objects
        .component_mut(handle)
        .and_then(|component| component.concrete.skin.as_mut())
        .context("Skin occurrence is missing its concrete state")?;
    skin.bone_transforms = vec![Mat2D::IDENTITY; skin.tendons.len() + 1];
    Ok(())
}

impl ArtboardInstance {
    /// Concrete `Skin::onDirty`: notify only the one retained Skinnable using
    /// that concrete owner's dirt family.
    pub(crate) fn runtime_skin_on_dirty(&mut self, handle: ComponentHandle) {
        let skinnable = self
            .objects
            .component(handle)
            .and_then(|component| component.concrete.skin.as_ref())
            .and_then(|skin| skin.skinnable);
        let Some(skinnable) = skinnable else {
            return;
        };
        match self
            .objects
            .component(skinnable)
            .and_then(|component| component.concrete.skinnable.as_ref())
            .map(|skinnable| skinnable.kind)
        {
            Some(RuntimeSkinnableKind::PointsPath) => {
                self.add_component_dirt(skinnable, ComponentDirt::PATH, false);
            }
            Some(RuntimeSkinnableKind::Mesh) => {
                self.add_component_dirt(skinnable, ComponentDirt::VERTICES, false);
            }
            _ => {}
        }
    }

    /// Concrete `Skin::update`: rebuild non-identity matrix slots from the
    /// retained Tendon/Bone relationships in insertion order.
    pub(crate) fn update_runtime_skin(&mut self, handle: ComponentHandle) {
        if self
            .objects
            .component(handle)
            .and_then(|component| component.concrete.skin.as_ref())
            .is_none()
        {
            return;
        }
        let tendon_count = self
            .objects
            .component(handle)
            .and_then(|component| component.concrete.skin.as_ref())
            .map_or(0, |skin| skin.tendons.len());
        for index in 0..tendon_count {
            let transform = self
                .objects
                .component(handle)
                .and_then(|component| component.concrete.skin.as_ref())
                .and_then(|skin| skin.tendons.get(index))
                .and_then(|tendon| self.objects.component(*tendon))
                .and_then(|tendon| tendon.concrete.tendon.as_ref())
                .and_then(|tendon| {
                    let bone = self.objects.component(tendon.bone?)?;
                    Some(bone.transform.world_transform.multiply(tendon.inverse_bind))
                });
            if let Some(transform) = transform
                && let Some(slot) = self
                    .objects
                    .component_mut(handle)
                    .and_then(|component| component.concrete.skin.as_mut())
                    .and_then(|skin| skin.bone_transforms.get_mut(index + 1))
            {
                *slot = transform;
            }
        }
        #[cfg(test)]
        if let Some(skin) = self
            .objects
            .component_mut(handle)
            .and_then(|component| component.concrete.skin.as_mut())
        {
            skin.buffer_rebuilds += 1;
        }
    }
}
