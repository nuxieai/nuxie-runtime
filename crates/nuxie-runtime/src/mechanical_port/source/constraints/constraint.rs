use crate::mechanical_port::source::{
    component::{Component, ComponentDirt},
    core_context::{CoreContext, StatusCode},
    generated::constraints::constraint_base::ConstraintBase,
    math::mat2d::Mat2D,
    transform_component::{TransformComponent, WorldTransformComponent},
};

pub trait Constraint: ConstraintBase {
    fn constrain(&mut self, component: &mut TransformComponent);

    fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let result = ConstraintBase::on_added_dirty(self, context);
        if !self.parent().is::<TransformComponent>() {
            return StatusCode::InvalidObject;
        }
        self.parent_mut()
            .as_mut::<TransformComponent>()
            .expect("type checked above")
            .add_constraint(self);
        result
    }

    fn mark_constraint_dirty(&mut self) {
        self.parent_mut()
            .as_mut::<TransformComponent>()
            .expect("Constraint parent was validated")
            .mark_transform_dirty();
    }

    fn strength_changed(&mut self) {
        self.mark_constraint_dirty();
    }

    fn build_dependencies(&mut self) {
        ConstraintBase::build_dependencies(self);
        let this = self.as_component_mut_ptr();
        self.parent_mut().add_dependent(this);
    }

    fn on_dirty(&mut self, _dirt: ComponentDirt) {
        self.mark_constraint_dirty();
    }
}

static IDENTITY: Mat2D = Mat2D::IDENTITY;

pub fn get_parent_world(component: &TransformComponent) -> &Mat2D {
    let parent: &Component = component.parent();
    if parent.is::<WorldTransformComponent>() {
        return parent
            .as_ref::<WorldTransformComponent>()
            .expect("type checked above")
            .world_transform();
    }
    &IDENTITY
}
