use crate::mechanical_port::source::{
    core_context::CoreContext, generated::bones::root_bone_base::RootBoneBase,
    status_code::StatusCode,
};

pub struct RootBone {
    pub base: RootBoneBase,
}

impl std::ops::Deref for RootBone {
    type Target = RootBoneBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for RootBone {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for RootBone {
    fn default() -> Self {
        Self {
            base: RootBoneBase::default(),
        }
    }
}

impl RootBone {
    pub fn x(&self) -> f32 {
        self.base.x()
    }

    pub fn set_x(&mut self, value: f32) {
        if !self.base.set_x_value(value) {
            return;
        }
        self.x_changed();
        self.base
            .base
            .core_mut()
            .notify_property_changed(RootBoneBase::X_PROPERTY_KEY);
    }

    pub fn y(&self) -> f32 {
        self.base.y()
    }

    pub fn set_y(&mut self, value: f32) {
        if !self.base.set_y_value(value) {
            return;
        }
        self.y_changed();
        self.base
            .base
            .core_mut()
            .notify_property_changed(RootBoneBase::Y_PROPERTY_KEY);
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        // Skip Bone::onAddedClean exactly: a root bone may have any
        // TransformComponent parent.
        crate::mechanical_port::source::transform_component::TransformComponent::on_added_clean(
            &mut self.base.base.base.base.base.base,
            context,
        )
    }

    pub fn x_changed(&mut self) {
        self.base.mark_transform_dirty();
    }

    pub fn y_changed(&mut self) {
        self.base.mark_transform_dirty();
    }
}
