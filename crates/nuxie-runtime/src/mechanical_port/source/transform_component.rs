use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    generated::core_registry::CoreCapabilities,
    generated::transform_component_base::{
        TransformComponentBase, TransformComponentBaseCallbacks,
    },
    intrinsically_sizeable::IntrinsicallySizeable,
    math::{aabb::Aabb, mat2d::Mat2D},
    status_code::StatusCode,
};

pub struct TransformComponent {
    pub base: TransformComponentBase,
    transform: Mat2D,
    render_opacity: f32,
    parent_transform_component: Option<CoreHandle>,
    constraints: Vec<CoreHandle>,
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
    pub fn set_scale_x(&mut self, value: f32) {
        if self.base.set_scale_x_value(value) {
            TransformComponentBaseCallbacks::scale_x_changed(self);
            TransformComponentBaseCallbacks::notify_property_changed(
                self,
                TransformComponentBase::SCALE_X_PROPERTY_KEY,
            );
        }
    }

    pub fn set_scale_y(&mut self, value: f32) {
        if self.base.set_scale_y_value(value) {
            TransformComponentBaseCallbacks::scale_y_changed(self);
            TransformComponentBaseCallbacks::notify_property_changed(
                self,
                TransformComponentBase::SCALE_Y_PROPERTY_KEY,
            );
        }
    }

    pub fn constraints(&self) -> &[CoreHandle] {
        &self.constraints
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        self.parent_transform_component =
            self.base
                .base
                .base
                .base
                .base
                .parent_handle()
                .filter(|parent| {
                    parent
                        .with(|parent| parent.as_world_transform_component().is_some())
                        .unwrap_or(false)
                });
        StatusCode::Ok
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        CoreCapabilities::component_collapse(self, value)
    }

    pub(crate) fn collapse_after_super(&mut self) {
        let dependents = self.base.base.base.base.base.dependents().to_vec();
        for dependent in dependents {
            let Some(dependent) = dependent.authored() else {
                continue;
            };
            dependent.with_mut(|dependent| {
                let is_constrained = dependent
                    .as_transform_component()
                    .is_some_and(|transform| !transform.constraints().is_empty());
                if is_constrained {
                    dependent.component_add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
                }
            });
        }
    }

    pub fn mark_dirty_if_constrained(&mut self) {
        if !self.constraints.is_empty() {
            CoreCapabilities::world_transform_mark_dirty(self);
        }
    }

    pub fn build_dependencies(&mut self) {
        let component = &mut self.base.base.base.base.base;
        let Some(this) = component.base.base.handle() else {
            return;
        };
        if let Some(parent) = component.parent_handle() {
            parent.with_mut(|parent| {
                parent.component_add_dependent(this);
            });
        }
    }

    pub fn mark_transform_dirty(&mut self) {
        CoreCapabilities::transform_mark_dirty(self);
    }

    pub(crate) fn mark_transform_dirty_occurrence(owner: &CoreHandle) {
        assert!(owner.is_type_of(TransformComponentBase::TYPE_KEY));
        let occurrence =
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            );
        // The recursive world-transform dirt visits child Paths, whose
        // callbacks can synchronously return to this component's Shape.
        if occurrence.add_dirt(ComponentDirt::TRANSFORM, false) {
            occurrence.add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
        }
    }

    pub(crate) fn mark_transform_dirty_from_shape(
        owner: &CoreHandle,
        active_shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
    ) {
        assert!(owner.is_type_of(TransformComponentBase::TYPE_KEY));
        let occurrence =
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            );
        if occurrence.add_dirt_from_shape(active_shape, ComponentDirt::TRANSFORM, false) {
            occurrence.add_dirt_from_shape(active_shape, ComponentDirt::WORLD_TRANSFORM, true);
        }
    }

    pub(crate) fn mark_transform_dirty_from_layout(
        owner: &CoreHandle,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        assert!(owner.is_type_of(TransformComponentBase::TYPE_KEY));
        let occurrence =
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            );
        if occurrence.add_dirt_from_layout(active, active_handle, ComponentDirt::TRANSFORM, false) {
            occurrence.add_dirt_from_layout(
                active,
                active_handle,
                ComponentDirt::WORLD_TRANSFORM,
                true,
            );
        }
    }

    pub fn update_transform_state(&mut self, x: f32, y: f32) {
        self.transform = Mat2D::from_rotation(self.base.rotation());
        self.transform[4] = x;
        self.transform[5] = y;
        self.transform
            .scale_by_values(self.base.scale_x(), self.base.scale_y());
    }

    pub fn compose_world_transform(&mut self) {
        let parent_transform = self.parent_transform_component.as_ref().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_world_transform_component()
                        .map(|parent| *parent.world_transform())
                })
                .flatten()
        });
        *self.base.base.mutable_world_transform() = parent_transform
            .map(|parent| parent * self.transform)
            .unwrap_or(self.transform);
    }

    pub fn apply_constraints(component: CoreHandle, constraints: Vec<CoreHandle>) {
        for constraint in constraints {
            constraint.with_mut(|constraint| {
                constraint.constraint_apply(component.clone());
            });
        }
    }

    pub fn update_render_opacity_state(&mut self, parent_child_opacity: Option<f32>) {
        self.render_opacity = self.base.base.base.opacity();
        if let Some(parent_child_opacity) = parent_child_opacity {
            self.render_opacity *= parent_child_opacity;
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

    pub fn rotation_changed(&mut self) {
        self.mark_transform_dirty();
    }
    pub fn scale_x_changed(&mut self) {
        self.mark_transform_dirty();
    }
    pub fn scale_y_changed(&mut self) {
        self.mark_transform_dirty();
    }

    pub fn add_constraint(&mut self, constraint: CoreHandle) {
        self.constraints.push(constraint);
    }

    pub fn constraint_bounds(&self) -> Aabb {
        Aabb::default()
    }

    pub fn local_bounds(&self) -> Aabb {
        Aabb::default()
    }

    pub fn parent_transform_component(&self) -> Option<CoreHandle> {
        self.parent_transform_component.clone()
    }
}

impl IntrinsicallySizeable for TransformComponent {}

impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
    for TransformComponent
{
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(
            &mut self.base,
            property_key,
        );
    }
}

impl crate::mechanical_port::source::generated::world_transform_component_base::WorldTransformComponentBaseCallbacks
    for TransformComponent
{
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(
            &mut self.base,
            property_key,
        );
    }

    fn opacity_changed(&mut self) {
        self.base.base.opacity_changed();
    }
}

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

impl std::ops::Deref for TransformComponent {
    type Target = TransformComponentBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransformComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
