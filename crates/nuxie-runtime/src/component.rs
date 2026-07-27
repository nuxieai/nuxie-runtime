use anyhow::{Context, Result};
use nuxie_graph::ComponentNode;

use super::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle, RuntimeComponent};
use crate::objects::InstanceObjectArena;

/// Read-only view of the concrete `Component` occurrences owned by one
/// Artboard instance.
///
/// Keeping this facade beside the Component implementation preserves the
/// public API while making `artboard.rs` a module entry point instead of the
/// owner of Component access.
#[derive(Clone, Copy)]
pub struct RuntimeComponents<'a> {
    pub(super) arena: &'a InstanceObjectArena,
}

impl<'a> RuntimeComponents<'a> {
    pub fn len(self) -> usize {
        self.arena.component_handles().len()
    }

    pub fn is_empty(self) -> bool {
        self.arena.component_handles().is_empty()
    }

    pub fn iter(self) -> impl Iterator<Item = &'a RuntimeComponent> + 'a {
        self.arena
            .component_handles()
            .iter()
            .filter_map(|handle| self.arena.component(*handle))
    }
}

impl ArtboardInstance {
    /// Attach one authored Component to its retained parent occurrence.
    ///
    /// This is the concrete Rust owner for `Component::validate` and the base
    /// portion of `Component::onAddedDirty`: every non-root Component must
    /// resolve a ContainerComponent parent, and the link is retained exactly
    /// once in authored order (`src/component.cpp:13-29`).
    pub(super) fn attach_component_parent(
        objects: &mut InstanceObjectArena,
        component: &ComponentNode,
        root: ComponentHandle,
        parent_key: u16,
    ) -> Result<ComponentHandle> {
        let handle = objects
            .component_handle(component.local_id)
            .context("authored Component handle is missing")?;
        if handle == root {
            return Ok(handle);
        }

        let parent_local = objects
            .uint_property(component.local_id, parent_key)
            .and_then(|parent| usize::try_from(parent).ok())
            .context("Component parentId does not resolve to an object slot")?;
        let parent = objects
            .component_handle(parent_local)
            .context("Component parentId does not resolve to a Component occurrence")?;
        let parent_type = objects
            .component(parent)
            .map(|component| component.type_name)
            .unwrap_or("<missing>");
        if !objects.is_container_component(parent) {
            anyhow::bail!(
                "Component {} local {} parent local {} type {} is not a ContainerComponent",
                component.type_name,
                component.local_id,
                parent_local,
                parent_type
            );
        }
        if !objects.link_parent(handle, parent) {
            anyhow::bail!("Component parent link could not be retained");
        }
        Ok(handle)
    }

    pub(crate) fn component_mut(&mut self, local_id: usize) -> Option<&mut RuntimeComponent> {
        self.objects.component_for_local_mut(local_id)
    }

    pub fn components(&self) -> RuntimeComponents<'_> {
        RuntimeComponents {
            arena: &self.objects,
        }
    }

    pub(crate) fn component_at(&self, handle: ComponentHandle) -> &RuntimeComponent {
        self.objects
            .component(handle)
            .expect("runtime component handle must address its occurrence")
    }

    pub(crate) fn component_at_mut(&mut self, handle: ComponentHandle) -> &mut RuntimeComponent {
        self.objects
            .component_mut(handle)
            .expect("runtime component handle must address its occurrence")
    }

    pub(crate) fn component_handle(&self, local_id: usize) -> Option<ComponentHandle> {
        self.objects.component_handle(local_id)
    }

    pub(crate) fn component_parent_handle(
        &self,
        handle: ComponentHandle,
    ) -> Option<ComponentHandle> {
        self.objects.component(handle)?.parent
    }

    pub(crate) fn component_local_id(&self, handle: ComponentHandle) -> Option<usize> {
        self.objects.component_local_id(handle)
    }

    pub(crate) fn component_parent_local(&self, local_id: usize) -> Option<usize> {
        let handle = self.component_handle(local_id)?;
        let parent = self.component_parent_handle(handle)?;
        self.objects.component_local_id(parent)
    }

    pub(crate) fn component_child_len(&self, handle: ComponentHandle) -> usize {
        self.objects.child_len(handle)
    }

    pub(crate) fn component_child_at(
        &self,
        handle: ComponentHandle,
        index: usize,
    ) -> Option<ComponentHandle> {
        self.objects.child_at(handle, index)
    }

    pub fn clear_component_dirt(&mut self, local_id: usize) {
        if let Some(component) = self.component_mut(local_id) {
            component.dirt = ComponentDirt::NONE;
        }
    }

    pub fn add_dirt(&mut self, local_id: usize, dirt: ComponentDirt, recurse: bool) -> bool {
        let Some(handle) = self.component_handle(local_id) else {
            return false;
        };
        self.add_component_dirt(handle, dirt, recurse)
    }

    /// Direct ownership port of `Component::addDirt`: retain the accumulated
    /// mask before callbacks, notify the Artboard, and recurse through the
    /// retained dependent list only when requested (`src/component.cpp:32-54`).
    pub(crate) fn add_component_dirt(
        &mut self,
        handle: ComponentHandle,
        dirt: ComponentDirt,
        recurse: bool,
    ) -> bool {
        if dirt.is_empty() {
            return false;
        }

        let Some(component) = self.objects.component(handle) else {
            return false;
        };
        if component.dirt.contains(dirt) {
            return false;
        }

        let accumulated = {
            let component = self
                .objects
                .component_mut(handle)
                .expect("component handle was resolved above");
            component.dirt |= dirt;
            component.dirt
        };
        self.dispatch_component_on_dirty(handle, accumulated);
        self.on_component_dirty_handle(handle);

        if recurse {
            let dependent_count = self.objects.dependent_len(handle);
            for index in 0..dependent_count {
                let Some(dependent) = self.objects.dependent_at(handle, index) else {
                    continue;
                };
                self.add_component_dirt(dependent, dirt, true);
            }
        }
        true
    }

    /// Direct base `Component::collapse` ownership: publish the Collapsed bit
    /// before callbacks, notify the Artboard, then update every retained
    /// collapsable in registration order (`src/component.cpp:76-95,117-127`).
    pub(super) fn collapse_component_base(
        &mut self,
        handle: ComponentHandle,
        collapsed: bool,
    ) -> bool {
        let Some(component) = self.objects.component(handle) else {
            return false;
        };
        if component.is_collapsed() == collapsed {
            return false;
        }

        let accumulated = {
            let component = self
                .objects
                .component_mut(handle)
                .expect("component handle was resolved above");
            if collapsed {
                component.dirt |= ComponentDirt::COLLAPSED;
            } else {
                component.dirt &= !ComponentDirt::COLLAPSED;
            }
            component.dirt
        };
        self.dispatch_component_on_dirty(handle, accumulated);
        self.on_component_dirty_handle(handle);
        let collapsable_count = self.objects.collapsable_len(handle);
        for index in 0..collapsable_count {
            let Some(data_bind) = self.objects.collapsable_at(handle, index) else {
                continue;
            };
            self.collapse_artboard_authored_data_bind(data_bind, collapsed);
        }
        true
    }

    /// C++ `Component::hitTestPoint` walks to the concrete parent while
    /// preserving `skipOnUnclipped` and clearing the primary-hit marker
    /// (`src/component.cpp:97-105`).
    pub(super) fn base_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        _is_primary_hit: bool,
    ) -> bool {
        let Some(parent) = self
            .objects
            .component(component)
            .and_then(|component| component.parent)
        else {
            return true;
        };
        self.component_hit_test_point(parent, position, skip_on_unclipped, false)
    }
}
