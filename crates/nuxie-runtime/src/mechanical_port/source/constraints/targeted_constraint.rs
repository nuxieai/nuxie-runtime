use crate::mechanical_port::source::{
    constraints::constraint::Constraint,
    core_context::{CoreContext, StatusCode},
    generated::constraints::targeted_constraint_base::TargetedConstraintBase,
    transform_component::TransformComponent,
};

pub struct TargetedConstraint {
    pub base: TargetedConstraintBase,
    pub target: Option<*mut TransformComponent>,
}

impl TargetedConstraint {
    pub fn new(base: TargetedConstraintBase) -> Self {
        Self { base, target: None }
    }

    pub fn requires_target(&self) -> bool {
        true
    }

    pub fn validate(&self, context: &CoreContext) -> bool {
        if !self.base.validate(context) {
            return false;
        }
        let core_object = context.resolve(self.base.target_id());
        if core_object.is_some_and(|object| !object.is::<TransformComponent>()) {
            return false;
        }
        !self.requires_target() || core_object.is_some()
    }

    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let core_object = context.resolve_mut(self.base.target_id());
        if self.requires_target() && core_object.is_none() {
            return StatusCode::MissingObject;
        }
        self.target = core_object.map(|object| object.cast_mut::<TransformComponent>());
        StatusCode::Ok
    }

    pub fn build_dependencies(&mut self) {
        if let Some(target) = self.target {
            // Raw pointer mirrors the non-owning C++ target link.
            unsafe { (*target).add_dependent(self.base.parent_mut()) };
        }
    }
}
