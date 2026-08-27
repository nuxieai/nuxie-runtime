use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    constraints::constraint::Constraint,
    core_context::CoreContext,
    generated::transform_component_base::{
        TransformComponentBase, TransformComponentBaseCallbacks,
    },
    intrinsically_sizeable::IntrinsicallySizeable,
    math::{aabb::Aabb, mat2d::Mat2D},
    status_code::StatusCode,
    world_transform_component::WorldTransformComponent,
};

pub struct TransformComponent {
    pub base: TransformComponentBase,
    transform: Mat2D,
    render_opacity: f32,
    parent_transform_component: Option<*mut WorldTransformComponent>,
    constraints: Vec<*mut dyn Constraint>,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            base: TransformComponentBase::default(),
            transform: Mat2D::default(),
            render_opacity: 0.0,
            parent_transform_component: None,
            constraints: Vec::new(),
        }
    }
}

impl TransformComponent {
    pub fn constraints(&self) -> &[*mut dyn Constraint] {
        &self.constraints
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        self.parent_transform_component = self
            .base
            .base
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_world_transform_component_mut())
            .map(|parent| parent as *mut WorldTransformComponent);
        StatusCode::Ok
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.base.base.base.base.collapse(value) {
            return false;
        }
        let dependents = self.base.base.base.base.base.dependents().to_vec();
        for dependent in dependents {
            if let Some(transform) = unsafe { &mut *dependent }.as_transform_component_mut() {
                transform.mark_dirty_if_constrained();
            }
        }
        true
    }

    pub fn mark_dirty_if_constrained(&mut self) {
        if !self.constraints.is_empty() {
            self.base
                .base
                .base
                .base
                .base
                .add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
        }
    }

    pub fn build_dependencies(&mut self) {
        let this = self as *mut TransformComponent;
        if let Some(parent) = self.base.base.base.base.parent_mut() {
            parent
                .base
                .base
                .add_dependent(this.cast::<crate::mechanical_port::source::component::Component>());
        }
    }

    pub fn mark_transform_dirty(&mut self) {
        if !self
            .base
            .base
            .base
            .base
            .base
            .add_dirt(ComponentDirt::TRANSFORM, false)
        {
            return;
        }
        self.base.base.mark_world_transform_dirty();
    }

    pub fn update_transform(&mut self) {
        self.transform = Mat2D::from_rotation(self.base.rotation());
        self.transform[4] = self.x();
        self.transform[5] = self.y();
        self.transform
            .scale_by_values(self.base.scale_x(), self.base.scale_y());
    }

    pub fn compose_world_transform(&mut self) {
        *self.base.base.mutable_world_transform() =
            if let Some(parent) = self.parent_transform_component {
                *unsafe { &*parent }.world_transform() * self.transform
            } else {
                self.transform
            };
    }

    pub fn update_world_transform(&mut self) {
        self.compose_world_transform();
        self.update_constraints();
    }

    pub fn update_constraints(&mut self) {
        for constraint in self.constraints.iter().copied() {
            unsafe { &mut *constraint }.constrain(self);
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if value.contains(ComponentDirt::TRANSFORM) {
            self.update_transform();
        }
        if value.contains(ComponentDirt::WORLD_TRANSFORM) {
            self.update_world_transform();
        }
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.render_opacity = self.base.base.base.opacity();
            if let Some(parent) = self.parent_transform_component {
                self.render_opacity *= unsafe { &*parent }.child_opacity();
            }
        }
    }

    pub fn child_opacity(&self) -> f32 {
        self.render_opacity
    }

    pub fn render_opacity(&self) -> f32 {
        self.render_opacity
    }

    pub fn transform(&self) -> &Mat2D {
        &self.transform
    }

    pub fn mutable_transform(&mut self) -> &mut Mat2D {
        &mut self.transform
    }

    pub fn x(&self) -> f32 {
        panic!("abstract TransformComponent::x");
    }

    pub fn y(&self) -> f32 {
        panic!("abstract TransformComponent::y");
    }

    pub fn rotation_changed(&mut self) {
        self.mark_transform_dirty();
    }
    pub fn scale_x_changed(&mut self) {
        self.mark_transform_dirty();
    }
    pub fn scale_y_changed(&mut self) {
        self.mark_transform_dirty();
    }

    pub fn add_constraint(&mut self, constraint: *mut dyn Constraint) {
        self.constraints.push(constraint);
    }

    pub fn constraint_bounds(&self) -> Aabb {
        Aabb::default()
    }

    pub fn local_bounds(&self) -> Aabb {
        Aabb::default()
    }

    pub fn parent_transform_component(&mut self) -> Option<&mut WorldTransformComponent> {
        self.parent_transform_component
            .map(|parent| unsafe { &mut *parent })
    }
}

impl IntrinsicallySizeable for TransformComponent {}

impl TransformComponentBaseCallbacks for TransformComponent {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
    fn rotation_changed(&mut self) {
        TransformComponent::rotation_changed(self);
    }
    fn scale_x_changed(&mut self) {
        TransformComponent::scale_x_changed(self);
    }
    fn scale_y_changed(&mut self) {
        TransformComponent::scale_y_changed(self);
    }
}
