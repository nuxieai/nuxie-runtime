//! Authored Node owner from pinned rive-runtime 160085c6 node.hpp/node.cpp.

use crate::mechanical_port::source::{
    artboard::Artboard,
    core::{CoreHandle, PropertySetterCompletion},
    generated::node_base::{NodeBase, NodeBaseCallbacks},
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    transform_component::TransformUpdate,
};

pub struct Node {
    pub base: NodeBase,
    local_transform: Mat2D,
    local_transform_needs_recompute: bool,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            base: NodeBase::default(),
            local_transform: Mat2D::default(),
            local_transform_needs_recompute: false,
        }
    }
}

crate::mechanical_port::source::transform_component::impl_transform_update!(
    Node,
    crate::mechanical_port::source::transform_component::update_transform_super::<Self>,
    crate::mechanical_port::source::transform_component::update_local_transform::<Self>,
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    crate::mechanical_port::source::transform_component::compose_world_transform::<Self>,
    crate::mechanical_port::source::transform_component::update_constraints_super::<Self>
);

impl Node {
    pub fn set_computed_local_x(&mut self, _value: f32) {}
    pub fn set_computed_local_y(&mut self, _value: f32) {}
    pub fn set_computed_world_x(&mut self, _value: f32) {}
    pub fn set_computed_world_y(&mut self, _value: f32) {}
    pub fn set_computed_root_x(&mut self, _value: f32) {}
    pub fn set_computed_root_y(&mut self, _value: f32) {}
    pub fn set_computed_width(&mut self, _value: f32) {}
    pub fn set_computed_height(&mut self, _value: f32) {}

    pub fn computed_local_x(&mut self) -> f32 {
        self.local_transform()[4]
    }

    pub fn computed_local_y(&mut self) -> f32 {
        self.local_transform()[5]
    }

    pub fn computed_world_x(&mut self) -> f32 {
        self.base.base.world_transform()[4]
    }

    pub fn computed_world_y(&mut self) -> f32 {
        self.base.base.world_transform()[5]
    }

    pub fn computed_root_x(&mut self) -> f32 {
        let Some(artboard) = self.base.artboard_handle() else {
            return 0.0;
        };
        let world = *self.base.base.world_transform();
        artboard
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.root_transform(Vec2D::new(world[4], world[5])).x
            })
            .expect("computedRootX requires a live artboard")
    }

    pub fn computed_root_y(&mut self) -> f32 {
        let Some(artboard) = self.base.artboard_handle() else {
            return 0.0;
        };
        let world = *self.base.base.world_transform();
        artboard
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.root_transform(Vec2D::new(world[4], world[5])).y
            })
            .expect("computedRootY requires a live artboard")
    }

    pub fn computed_width(&mut self) -> f32 {
        0.0
    }

    pub fn computed_height(&mut self) -> f32 {
        0.0
    }

    /// Node's cache invalidation precedes the most-derived composition and
    /// constraints, as in Node::updateWorldTransform and its superclass.
    pub(crate) fn update_world_transform_occurrence<T: TransformUpdate>(owner: &CoreHandle) {
        if owner
            .with_downcast_mut::<T, _>(|object| {
                object
                    .as_node_mut()
                    .expect("Node virtual receiver")
                    .update_world_transform_before_super();
                // This prefix only invalidates a matrix cache; composition can
                // share the same receiver loan. Constraints can reenter it.
                T::compose_world_transform(object);
            })
            .is_some()
        {
            T::update_constraints(owner);
        }
    }

    pub(crate) fn update_world_transform_before_super(&mut self) {
        self.local_transform_needs_recompute = true;
    }

    pub fn local_transform(&mut self) -> Mat2D {
        if self.local_transform_needs_recompute {
            self.local_transform_needs_recompute = false;
            if let Some(parent) = self.base.base.parent_transform_component() {
                let parent_world = parent
                    .with(|parent| {
                        *parent
                            .as_world_transform_component()
                            .expect("parent transform")
                            .world_transform()
                    })
                    .expect("live parent transform");
                let mut inverse = Mat2D::default();
                if parent_world.invert(&mut inverse) {
                    self.local_transform = inverse * *self.base.base.world_transform();
                    return self.local_transform;
                }
            }
            self.local_transform = Mat2D::default();
        }
        self.local_transform
    }

    pub fn x_changed(&mut self) {
        self.base.base.mark_transform_dirty();
    }

    pub fn y_changed(&mut self) {
        self.base.base.mark_transform_dirty();
    }

    pub fn mark_layout_node_dirty(&mut self) {
        let mut parent = self.base.parent_handle();
        while let Some(current) = parent {
            let is_layout = current
                .with(|current| current.as_layout_component().is_some())
                .expect("a Node ancestor remains live");
            if is_layout {
                LayoutComponent::mark_layout_node_dirty_occurrence(&current, false);
            }
            // The source for-loop reads p->parent() after marking this ancestor.
            // Its synchronous callbacks may have reparented the same ancestor.
            parent = current
                .with(|current| {
                    current
                        .as_component()
                        .and_then(|component| component.parent_handle())
                })
                .expect("a Node ancestor remains live");
        }
    }

    // NodeBase's generated x/y setter order, with the existing safe-Rust
    // notification completion boundary retained for occurrence callers.
    pub fn set_x(&mut self, value: f32) {
        let mut completion = PropertySetterCompletion::default();
        self.set_x_with_completion(value, &mut completion);
        completion.finish();
    }

    pub(crate) fn set_x_with_completion(
        &mut self,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        if self.base.set_x_value(value) {
            self.x_changed();
            completion.record(&self.base, NodeBase::X_PROPERTY_KEY);
        }
    }

    pub fn set_y(&mut self, value: f32) {
        let mut completion = PropertySetterCompletion::default();
        self.set_y_with_completion(value, &mut completion);
        completion.finish();
    }

    pub(crate) fn set_y_with_completion(
        &mut self,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        if self.base.set_y_value(value) {
            self.y_changed();
            completion.record(&self.base, NodeBase::Y_PROPERTY_KEY);
        }
    }
}

impl NodeBaseCallbacks for Node {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
    fn x_changed(&mut self) {
        Node::x_changed(self);
    }
    fn y_changed(&mut self) {
        Node::y_changed(self);
    }
    fn set_computed_local_x(&mut self, value: f32) {
        Node::set_computed_local_x(self, value);
    }
    fn computed_local_x(&mut self) -> f32 {
        Node::computed_local_x(self)
    }
    fn set_computed_local_y(&mut self, value: f32) {
        Node::set_computed_local_y(self, value);
    }
    fn computed_local_y(&mut self) -> f32 {
        Node::computed_local_y(self)
    }
    fn set_computed_world_x(&mut self, value: f32) {
        Node::set_computed_world_x(self, value);
    }
    fn computed_world_x(&mut self) -> f32 {
        Node::computed_world_x(self)
    }
    fn set_computed_world_y(&mut self, value: f32) {
        Node::set_computed_world_y(self, value);
    }
    fn computed_world_y(&mut self) -> f32 {
        Node::computed_world_y(self)
    }
    fn set_computed_root_x(&mut self, value: f32) {
        Node::set_computed_root_x(self, value);
    }
    fn computed_root_x(&mut self) -> f32 {
        Node::computed_root_x(self)
    }
    fn set_computed_root_y(&mut self, value: f32) {
        Node::set_computed_root_y(self, value);
    }
    fn computed_root_y(&mut self) -> f32 {
        Node::computed_root_y(self)
    }
    fn set_computed_width(&mut self, value: f32) {
        Node::set_computed_width(self, value);
    }
    fn computed_width(&mut self) -> f32 {
        Node::computed_width(self)
    }
    fn set_computed_height(&mut self, value: f32) {
        Node::set_computed_height(self, value);
    }
    fn computed_height(&mut self) -> f32 {
        Node::computed_height(self)
    }
}

impl std::ops::Deref for Node {
    type Target = NodeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
