use crate::mechanical_port::source::{
    component::Component,
    core::CoreHandle,
    core_context::CoreContext,
    generated::bones::bone_base::{BoneBase, BoneBaseCallbacks},
    math::vec2d::Vec2D,
    status_code::StatusCode,
    transform_component::TransformComponent,
};

struct SilentBoneCallbacks;
impl BoneBaseCallbacks for SilentBoneCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}

pub struct Bone {
    pub base: BoneBase,
    child_bones: Vec<CoreHandle>,
    peer_constraints: Vec<CoreHandle>,
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            base: BoneBase::default(),
            child_bones: Vec::new(),
            peer_constraints: Vec::new(),
        }
    }
}

impl Bone {
    fn component(&self) -> &Component {
        &self.base.base.base.base.base.base.base.base.base.base
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.base.base.base.base.base.base.base.base.base.base
    }

    pub(crate) fn core_mut(&mut self) -> &mut crate::mechanical_port::source::core::Core {
        &mut self.component_mut().base.base
    }

    fn transform_component(&self) -> &TransformComponent {
        &self.base.base.base.base
    }

    fn transform_component_mut(&mut self) -> &mut TransformComponent {
        &mut self.base.base.base.base
    }

    pub fn child_bones(&self) -> Vec<CoreHandle> {
        // The C++ signature returns its vector by value.
        self.child_bones.clone()
    }

    pub fn add_child_bone(&mut self, bone: CoreHandle) {
        self.child_bones.push(bone);
    }

    pub fn on_added_clean(
        &mut self,
        this: CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        // The pinned owner deliberately ignores the superclass status here.
        let _ = self.transform_component_mut().on_added_clean(context);
        let Some(parent) = context.resolve(self.component().base.parent_id()) else {
            return StatusCode::MissingObject;
        };
        if !parent.is_type_of(BoneBase::TYPE_KEY) {
            return StatusCode::MissingObject;
        }
        parent
            .with_downcast_mut::<Bone, _>(|bone| bone.add_child_bone(this))
            .map_or(StatusCode::MissingObject, |_| StatusCode::Ok)
    }

    pub fn length(&self) -> f32 {
        self.base.length()
    }

    pub fn set_length(&mut self, value: f32) {
        if self.base.length() == value {
            return;
        }
        let mut callbacks = SilentBoneCallbacks;
        self.base.set_length(value, &mut callbacks);
        self.length_changed();
        self.core_mut()
            .notify_property_changed(BoneBase::LENGTH_PROPERTY_KEY);
    }

    pub fn length_changed(&mut self) {
        for child in self.child_bones.iter().cloned() {
            child
                .with_downcast_mut::<Bone, _>(|bone| {
                    bone.transform_component_mut().mark_transform_dirty()
                })
                .expect("a retained child Bone must remain a Bone");
        }
    }

    pub fn x(&self, context: &dyn CoreContext) -> f32 {
        let parent = context
            .resolve(self.component().base.parent_id())
            .expect("onAddedClean requires a Bone parent");
        parent
            .with_downcast::<Bone, _>(|bone| bone.base.length())
            .expect("onAddedClean requires a Bone parent")
    }

    pub fn y(&self) -> f32 {
        0.0
    }

    pub fn tip_world_translation(&self) -> Vec2D {
        self.transform_component()
            .base
            .base
            .world_transform()
            .transform_point(Vec2D::new(self.base.length(), 0.0))
    }

    pub fn add_peer_constraint(&mut self, peer: CoreHandle) {
        assert!(!self.peer_constraints.contains(&peer));
        self.peer_constraints.push(peer);
    }

    pub fn peer_constraints(&self) -> &[CoreHandle] {
        &self.peer_constraints
    }
}
impl crate::mechanical_port::source::generated::bones::bone_base::BoneBaseCallbacks for Bone {
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
