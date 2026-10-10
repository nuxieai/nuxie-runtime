use std::{cell::Cell, rc::Rc};

use crate::mechanical_port::source::{
    generated::core_registry::CoreCapabilities,
    generated::world_transform_component_base::{
        WorldTransformComponentBase, WorldTransformComponentBaseCallbacks,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

#[derive(Default)]
pub struct WorldTransformComponent {
    pub base: WorldTransformComponentBase,
    // The world transform, and the TransformComponent state built on it,
    // shared with the path pipeline's cells (see TransformCells).
    cells: Rc<TransformCells>,
}

/// A transform component's derived state: its world and local transforms,
/// render opacity and whether constraints apply. Shared so the dirt cascade,
/// the path pipeline and child transforms read it without borrowing the
/// component.
#[derive(Default)]
pub(crate) struct TransformCells {
    world: Cell<Mat2D>,
    local: Cell<Mat2D>,
    // None until the first RenderOpacity update.
    render_opacity: Cell<Option<f32>>,
    has_constraints: Cell<bool>,
}

impl TransformCells {
    #[inline]
    pub(crate) fn world(&self) -> Mat2D {
        self.world.get()
    }

    #[inline]
    pub(crate) fn set_world(&self, world: Mat2D) {
        self.world.set(world);
    }

    #[inline]
    pub(crate) fn local(&self) -> Mat2D {
        self.local.get()
    }

    #[inline]
    pub(crate) fn set_local(&self, local: Mat2D) {
        self.local.set(local);
    }

    #[inline]
    pub(crate) fn render_opacity(&self) -> f32 {
        self.render_opacity.get().unwrap_or(0.0)
    }

    pub(crate) fn computed_render_opacity(&self) -> Option<f32> {
        self.render_opacity.get()
    }

    pub(crate) fn set_render_opacity(&self, opacity: f32) {
        self.render_opacity.set(Some(opacity));
    }

    #[inline]
    pub(crate) fn has_constraints(&self) -> bool {
        self.has_constraints.get()
    }

    pub(crate) fn set_has_constraints(&self) {
        self.has_constraints.set(true);
    }
}

impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
    for WorldTransformComponent
{
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(
            &mut self.base,
            property_key,
        );
    }
}

impl WorldTransformComponentBaseCallbacks for WorldTransformComponent {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn opacity_changed(&mut self) {
        WorldTransformComponent::opacity_changed(self);
    }
}

impl WorldTransformComponent {
    pub fn child_opacity(&self) -> f32 {
        self.base.opacity()
    }

    pub fn mark_world_transform_dirty(&mut self) {
        CoreCapabilities::world_transform_mark_dirty(self);
    }

    pub fn world_transform(&self) -> Mat2D {
        self.cells.world()
    }

    /// Edit the world transform in place.
    pub fn with_world_transform_mut<R>(&mut self, edit: impl FnOnce(&mut Mat2D) -> R) -> R {
        let mut world = self.cells.world();
        let result = edit(&mut world);
        self.cells.set_world(world);
        result
    }

    pub fn set_world_transform(&mut self, transform: Mat2D) {
        self.cells.set_world(transform);
    }

    pub fn world_translation(&self) -> Vec2D {
        self.cells.world().translation()
    }

    pub(crate) fn cells(&self) -> &Rc<TransformCells> {
        &self.cells
    }

    pub fn opacity_changed(&mut self) {
        CoreCapabilities::world_transform_opacity_changed(self);
    }
}

impl std::ops::Deref for WorldTransformComponent {
    type Target = WorldTransformComponentBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for WorldTransformComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
