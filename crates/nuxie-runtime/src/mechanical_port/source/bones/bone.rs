use crate::mechanical_port::source::{
    component::ComponentHandle, core_context::CoreContext, generated::bones::bone_base::BoneBase,
    math::vec2d::Vec2D, status_code::StatusCode,
};

pub struct Bone {
    pub base: BoneBase,
    child_bones: Vec<ComponentHandle>,
    peer_constraints: Vec<ComponentHandle>,
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
    pub fn child_bones(&self) -> Vec<ComponentHandle> {
        // The C++ signature returns its vector by value.
        self.child_bones.clone()
    }

    pub fn add_child_bone(&mut self, bone: ComponentHandle) {
        self.child_bones.push(bone);
    }

    pub fn on_added_clean(
        &mut self,
        this: ComponentHandle,
        context: &mut CoreContext,
    ) -> StatusCode {
        // The pinned owner deliberately ignores the superclass status here.
        let _ = self.base.on_added_clean(context);
        let Some(parent) = self.base.parent() else {
            return StatusCode::MissingObject;
        };
        if !context.is_bone(parent) {
            return StatusCode::MissingObject;
        }
        context
            .bone_mut(parent)
            .expect("a component classified as Bone must resolve as Bone")
            .add_child_bone(this);
        StatusCode::Ok
    }

    pub fn length_changed(&mut self, context: &mut CoreContext) {
        for child in self.child_bones.iter().copied() {
            context
                .transform_component_mut(child)
                .expect("a retained child Bone must remain a transform component")
                .mark_transform_dirty();
        }
    }

    pub fn x(&self, context: &CoreContext) -> f32 {
        let parent = self
            .base
            .parent()
            .expect("onAddedClean requires a Bone parent");
        context
            .bone(parent)
            .expect("onAddedClean requires a Bone parent")
            .base
            .length()
    }

    pub fn y(&self) -> f32 {
        0.0
    }

    pub fn tip_world_translation(&self) -> Vec2D {
        self.base
            .world_transform()
            .transform_point(Vec2D::new(self.base.length(), 0.0))
    }

    pub fn add_peer_constraint(&mut self, peer: ComponentHandle) {
        assert!(!self.peer_constraints.contains(&peer));
        self.peer_constraints.push(peer);
    }

    pub fn peer_constraints(&self) -> &[ComponentHandle] {
        &self.peer_constraints
    }
}
