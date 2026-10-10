use std::{cell::Cell, rc::Rc};

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    generated::core_registry::CoreCapabilities,
    generated::world_transform_component_base::{
        WorldTransformComponentBase, WorldTransformComponentBaseCallbacks,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

#[derive(Default)]
pub struct WorldTransformComponent {
    pub base: WorldTransformComponentBase,
}

/// A transform component's inputs (opacity, rotation, scale, and x/y or a
/// bone's length) and derived state (world and local transforms, render
/// opacity, whether constraints apply). The generated bases store their
/// properties here, and the dirt cascade, the path pipeline and child
/// transforms read it without borrowing the component.
pub(crate) struct TransformCells {
    opacity: Cell<f32>,
    rotation: Cell<f32>,
    scale_x: Cell<f32>,
    scale_y: Cell<f32>,
    // Node and RootBone positions; a Bone's x is its parent's length.
    x: Cell<f32>,
    y: Cell<f32>,
    length: Cell<f32>,
    world: Cell<Mat2D>,
    local: Cell<Mat2D>,
    // None until the first RenderOpacity update.
    render_opacity: Cell<Option<f32>>,
    // An Artboard's opacity in its host; children's opacity includes it.
    host_opacity: Cell<f32>,
    has_constraints: Cell<bool>,
    // A Node's cached local transform must be recomputed from its world.
    node_local_stale: Cell<bool>,
}

impl Default for TransformCells {
    fn default() -> Self {
        Self {
            opacity: Cell::new(1.0),
            rotation: Cell::new(0.0),
            scale_x: Cell::new(1.0),
            scale_y: Cell::new(1.0),
            x: Cell::new(0.0),
            y: Cell::new(0.0),
            length: Cell::new(0.0),
            world: Cell::new(Mat2D::default()),
            local: Cell::new(Mat2D::default()),
            render_opacity: Cell::new(None),
            host_opacity: Cell::new(1.0),
            has_constraints: Cell::new(false),
            node_local_stale: Cell::new(false),
        }
    }
}

/// Where a plain transform's local translation comes from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Translation {
    /// A Node's or RootBone's own x and y.
    Own,
    /// A Bone's: its parent bone's length along x.
    ParentLength,
}

/// TransformComponent::update for a Node, Bone or RootBone without
/// constraints. Its inputs, and its parent's world transform and child
/// opacity, are all cells, so the update borrows nothing.
pub(crate) struct PlainTransform {
    pub(crate) cells: Rc<TransformCells>,
    // The parent TransformComponent's handle (for liveness) and cells.
    pub(crate) parent: Option<(CoreHandle, Rc<TransformCells>)>,
    pub(crate) translation: Translation,
    // Node::updateWorldTransform first marks its cached local transform stale.
    pub(crate) node: bool,
    // The type's own work after TransformComponent::update (a Shape's opacity
    // propagation to its paints).
    pub(crate) after: Option<fn(&CoreHandle, ComponentDirt)>,
}

impl PlainTransform {
    pub(crate) fn update(&self, owner: &CoreHandle, dirt: ComponentDirt) {
        self.update_transform(dirt);
        if let Some(after) = self.after {
            after(owner, dirt);
        }
    }

    fn update_transform(&self, dirt: ComponentDirt) {
        let parent = self
            .parent
            .as_ref()
            .filter(|(handle, _)| handle.is_occupied())
            .map(|(_, cells)| &**cells);
        let cells = &*self.cells;
        if dirt.contains(ComponentDirt::TRANSFORM) {
            let (x, y) = match self.translation {
                Translation::Own => (cells.x(), cells.y()),
                Translation::ParentLength => (
                    parent
                        .expect("onAddedClean requires a Bone parent")
                        .length(),
                    0.0,
                ),
            };
            let mut transform = Mat2D::from_rotation(cells.rotation());
            transform[4] = x;
            transform[5] = y;
            transform.scale_by_values(cells.scale_x(), cells.scale_y());
            cells.set_local(transform);
        }
        if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
            if self.node {
                cells.mark_node_local_stale();
            }
            let local = cells.local();
            cells.set_world(match parent {
                Some(parent) => parent.world() * local,
                None => local,
            });
        }
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            let mut opacity = cells.opacity();
            if let Some(parent) = parent {
                opacity *= parent.child_opacity();
            }
            cells.set_render_opacity(opacity);
        }
    }
}

macro_rules! input_cells {
    ($($get:ident, $set:ident;)*) => {
        $(
            #[inline]
            pub(crate) fn $get(&self) -> f32 {
                self.$get.get()
            }

            #[inline]
            pub(crate) fn $set(&self, value: f32) {
                self.$get.set(value);
            }
        )*
    };
}

impl TransformCells {
    input_cells! {
        opacity, set_opacity;
        rotation, set_rotation;
        scale_x, set_scale_x;
        scale_y, set_scale_y;
        x, set_x;
        y, set_y;
        length, set_length;
        host_opacity, set_host_opacity;
    }

    pub(crate) fn mark_node_local_stale(&self) {
        self.node_local_stale.set(true);
    }

    pub(crate) fn take_node_local_stale(&self) -> bool {
        self.node_local_stale.replace(false)
    }

    /// TransformComponent::childOpacity(): the render opacity, times an
    /// Artboard's host opacity (1 for everything else, which is exact).
    #[inline]
    pub(crate) fn child_opacity(&self) -> f32 {
        self.render_opacity() * self.host_opacity.get()
    }

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
        self.cells().world()
    }

    /// Edit the world transform in place.
    pub fn with_world_transform_mut<R>(&mut self, edit: impl FnOnce(&mut Mat2D) -> R) -> R {
        let mut world = self.cells().world();
        let result = edit(&mut world);
        self.cells().set_world(world);
        result
    }

    pub fn set_world_transform(&mut self, transform: Mat2D) {
        self.cells().set_world(transform);
    }

    pub fn world_translation(&self) -> Vec2D {
        self.cells().world().translation()
    }

    pub(crate) fn cells(&self) -> &Rc<TransformCells> {
        self.base.cells()
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
