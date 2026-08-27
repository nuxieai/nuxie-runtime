use crate::mechanical_port::source::{
    core_context::CoreContext, generated::bones::root_bone_base::RootBoneBase,
    status_code::StatusCode,
};

pub struct RootBone {
    pub base: RootBoneBase,
}

impl Default for RootBone {
    fn default() -> Self {
        Self {
            base: RootBoneBase::default(),
        }
    }
}

impl RootBone {
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        // Skip Bone::onAddedClean exactly: a root bone may have any
        // TransformComponent parent.
        self.base.transform_component_on_added_clean(context)
    }

    pub fn x_changed(&mut self) {
        self.base.mark_transform_dirty();
    }

    pub fn y_changed(&mut self) {
        self.base.mark_transform_dirty();
    }
}
