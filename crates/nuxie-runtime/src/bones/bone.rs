//! Direct Rust owner for pinned C++ `include/rive/bones/bone.hpp` and
//! `src/bones/bone.cpp`.
//!
//! The authored generated fields remain in the occurrence's object storage.
//! This module owns Bone's runtime child/peer relationships and its concrete
//! callbacks. Transform propagation and IK orchestration remain on their
//! corresponding owners.

use anyhow::Context;

use crate::artboard::ArtboardInstance;
use crate::components::ComponentHandle;
use crate::objects::InstanceObjectArena;
use crate::properties::property_key_for_name;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBoneState {
    /// Concrete C++ subtype identity used by the `Bone::x/y` versus
    /// `RootBoneBase::x/y` virtual dispatch.
    pub(crate) is_root: bool,
    pub(crate) child_bones: Vec<ComponentHandle>,
    pub(crate) peer_constraints: Vec<ComponentHandle>,
}

impl RuntimeBoneState {
    pub(crate) fn new(is_root: bool) -> Self {
        Self {
            is_root,
            child_bones: Vec::new(),
            peer_constraints: Vec::new(),
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::new(self.is_root)
    }
}

/// Concrete `Bone::onAddedClean`: a non-root Bone requires a Bone parent and
/// registers in that parent's authored-order child list.
pub(crate) fn on_added_clean(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> anyhow::Result<()> {
    let parent = objects
        .component(handle)
        .and_then(|component| component.parent)
        .context("Bone is missing its parent Component")?;
    let Some(parent_bone) = objects
        .component_mut(parent)
        .and_then(|parent| parent.concrete.bone.as_mut())
    else {
        anyhow::bail!("Bone parent is not a Bone");
    };
    parent_bone.child_bones.push(handle);
    Ok(())
}

/// Literal `Bone::addPeerConstraint`: authored construction must not register
/// the same peer twice.
pub(crate) fn add_peer_constraint(
    objects: &mut InstanceObjectArena,
    bone: ComponentHandle,
    peer: ComponentHandle,
) {
    let peers = &mut objects
        .component_mut(bone)
        .expect("IK ancestor Bone was validated")
        .concrete
        .bone
        .as_mut()
        .expect("IK ancestor owns Bone state")
        .peer_constraints;
    assert!(
        !peers.contains(&peer),
        "C++ Bone::addPeerConstraint requires unique IK registration"
    );
    peers.push(peer);
}

impl ArtboardInstance {
    /// Concrete `Bone::x/y` dispatch. RootBone retains its authored x/y;
    /// ordinary Bone derives x from its retained Bone parent's length and y
    /// from the fixed zero returned by C++.
    pub(crate) fn runtime_bone_authored_translation(&self, local_id: usize) -> Option<(f32, f32)> {
        let handle = self.component_handle(local_id)?;
        let component = self.objects.component(handle)?;
        if component
            .concrete
            .bone
            .as_ref()
            .is_none_or(|bone| bone.is_root)
        {
            return None;
        }
        let parent_length = component
            .parent
            .and_then(|parent| self.objects.component_local_id(parent))
            .and_then(|parent_local| self.bone_length(parent_local))
            .unwrap_or(0.0);
        Some((parent_length, 0.0))
    }

    pub(crate) fn bone_length(&self, local_id: usize) -> Option<f32> {
        self.component(local_id)?.concrete.bone.as_ref()?;
        self.objects
            .double_property_by_name(local_id, "length")
            .or(Some(0.0))
    }

    /// Concrete `Bone::lengthChanged`: mark only the retained child Bones'
    /// transforms dirty, in child insertion order.
    pub(crate) fn apply_bone_double_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        if self
            .component(local_id)
            .and_then(|component| component.concrete.bone.as_ref())
            .is_none()
            || property_key_for_name("Bone", "length") != Some(property_key)
        {
            return false;
        }
        let Some(handle) = self.component_handle(local_id) else {
            return false;
        };
        let child_count = self
            .objects
            .component(handle)
            .and_then(|component| component.concrete.bone.as_ref())
            .map_or(0, |bone| bone.child_bones.len());
        for index in 0..child_count {
            let child = self
                .objects
                .component(handle)
                .and_then(|component| component.concrete.bone.as_ref())
                .and_then(|bone| bone.child_bones.get(index))
                .copied();
            if let Some(child) = child {
                self.mark_transform_dirty_handle(child);
            }
        }
        true
    }
}
