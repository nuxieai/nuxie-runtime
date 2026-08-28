use crate::mechanical_port::source::{
    generated::node_base::{NodeBase, NodeBaseCallbacks},
    math::{mat2d::Mat2D, vec2d::Vec2D},
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
        let world = *self.base.base.world_transform();
        self.base
            .base
            .artboard_mut()
            .map(|artboard| artboard.root_transform(Vec2D::new(world[4], world[5])).x)
            .unwrap_or(world[4])
    }

    pub fn computed_root_y(&mut self) -> f32 {
        let world = *self.base.base.world_transform();
        self.base
            .base
            .artboard_mut()
            .map(|artboard| artboard.root_transform(Vec2D::new(world[4], world[5])).y)
            .unwrap_or(world[5])
    }

    pub fn computed_width(&mut self) -> f32 {
        0.0
    }

    pub fn computed_height(&mut self) -> f32 {
        0.0
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
        let mut parent = self.base.base.parent_mut();
        while let Some(current) = parent {
            if let Some(layout) = current.as_layout_component_mut() {
                layout.mark_layout_node_dirty();
            }
            parent = current.base.base.parent_mut();
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
