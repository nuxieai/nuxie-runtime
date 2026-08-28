use crate::mechanical_port::source::{
    core_context::CoreContext,
    generated::bones::{
        bone_base::BoneBaseCallbacks,
        root_bone_base::{RootBoneBase, RootBoneBaseCallbacks},
    },
    status_code::StatusCode,
};

struct SilentRootBoneCallbacks;
impl BoneBaseCallbacks for SilentRootBoneCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
impl RootBoneBaseCallbacks for SilentRootBoneCallbacks {}

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
    pub fn x(&self) -> f32 {
        self.base.x()
    }

    pub fn set_x(&mut self, value: f32) {
        if self.base.x() == value {
            return;
        }
        let mut callbacks = SilentRootBoneCallbacks;
        self.base.set_x(value, &mut callbacks);
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
        if self.base.y() == value {
            return;
        }
        let mut callbacks = SilentRootBoneCallbacks;
        self.base.set_y(value, &mut callbacks);
        self.y_changed();
        self.base
            .base
            .core_mut()
            .notify_property_changed(RootBoneBase::Y_PROPERTY_KEY);
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
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
