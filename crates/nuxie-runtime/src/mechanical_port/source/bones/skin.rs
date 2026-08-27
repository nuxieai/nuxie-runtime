use crate::mechanical_port::source::{
    component::{ComponentDirt, ComponentHandle},
    core_context::CoreContext,
    generated::bones::skin_base::SkinBase,
    math::mat2d::Mat2D,
    status_code::StatusCode,
};

use super::skinnable;

pub struct Skin {
    pub base: SkinBase,
    world_transform: Mat2D,
    tendons: Vec<ComponentHandle>,
    bone_transforms: Option<Vec<f32>>,
    skinnable: Option<ComponentHandle>,
}

impl Default for Skin {
    fn default() -> Self {
        Self {
            base: SkinBase::default(),
            world_transform: Mat2D::identity(),
            tendons: Vec::new(),
            bone_transforms: None,
            skinnable: None,
        }
    }
}

impl Skin {
    pub(crate) fn add_tendon(&mut self, tendon: ComponentHandle) {
        self.tendons.push(tendon);
    }

    pub fn on_added_dirty(
        &mut self,
        this: ComponentHandle,
        context: &mut CoreContext,
    ) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.world_transform = Mat2D::new(
            self.base.xx(),
            self.base.xy(),
            self.base.yx(),
            self.base.yy(),
            self.base.tx(),
            self.base.ty(),
        );

        let Some(parent) = self.base.parent() else {
            return StatusCode::MissingObject;
        };
        let Some(skinnable) = skinnable::from(parent, context) else {
            return StatusCode::MissingObject;
        };
        context
            .skinnable_mut(skinnable)
            .expect("Skinnable::from accepted only concrete skinnable types")
            .set_skin(this);
        self.skinnable = Some(skinnable);
        StatusCode::Ok
    }

    pub fn update(&mut self, _value: ComponentDirt, context: &CoreContext) {
        let bone_transforms = self
            .bone_transforms
            .as_mut()
            .expect("buildDependencies initializes the bone transform buffer");
        let mut transform_index = 6;
        for tendon_handle in self.tendons.iter().copied() {
            let tendon = context
                .tendon(tendon_handle)
                .expect("a retained Tendon must remain a Tendon");
            let bone_handle = tendon
                .bone()
                .expect("Tendon::onAddedDirty resolves its Bone before update");
            let bone = context
                .bone(bone_handle)
                .expect("a Tendon Bone handle must remain a Bone");
            let world = bone.base.world_transform().multiply(tendon.inverse_bind());
            for coefficient in world.values() {
                bone_transforms[transform_index] = *coefficient;
                transform_index += 1;
            }
        }
    }

    pub fn build_dependencies(&mut self, this: ComponentHandle, context: &mut CoreContext) {
        for tendon_handle in self.tendons.iter().copied() {
            let tendon = context
                .tendon(tendon_handle)
                .expect("a retained Tendon must remain a Tendon");
            let bone_handle = tendon
                .bone()
                .expect("Tendon::onAddedDirty resolves its Bone before dependency building");
            let peer_constraints = context
                .bone(bone_handle)
                .expect("a Tendon Bone handle must remain a Bone")
                .peer_constraints()
                .to_vec();
            context
                .component_mut(bone_handle)
                .expect("a Tendon Bone handle must remain a Component")
                .add_dependent(this);
            for constraint_handle in peer_constraints {
                let constraint_parent = context
                    .component(constraint_handle)
                    .expect("a retained Constraint must remain a Component")
                    .parent()
                    .expect("a peer Constraint must have a parent");
                context
                    .component_mut(constraint_parent)
                    .expect("a Constraint parent must remain a Component")
                    .add_dependent(this);
            }
        }

        assert!(self.bone_transforms.is_none());
        let mut bone_transforms = vec![0.0; (self.tendons.len() + 1) * 6];
        bone_transforms[0] = 1.0;
        bone_transforms[1] = 0.0;
        bone_transforms[2] = 0.0;
        bone_transforms[3] = 1.0;
        bone_transforms[4] = 0.0;
        bone_transforms[5] = 0.0;
        self.bone_transforms = Some(bone_transforms);
    }

    pub fn deform(&self, vertices: &[ComponentHandle], context: &mut CoreContext) {
        let bone_transforms = self
            .bone_transforms
            .as_deref()
            .expect("buildDependencies initializes the bone transform buffer");
        for vertex_handle in vertices.iter().copied() {
            context
                .vertex_mut(vertex_handle)
                .expect("a retained Vertex must remain a Vertex")
                .deform(&self.world_transform, bone_transforms);
        }
    }

    pub fn on_dirty(&mut self, _dirt: ComponentDirt, context: &mut CoreContext) {
        if let Some(skinnable) = self.skinnable {
            context
                .skinnable_mut(skinnable)
                .expect("the retained Skinnable must remain live")
                .mark_skin_dirty();
        }
    }

    #[cfg(feature = "testing")]
    pub fn tendons_mut(&mut self) -> &mut Vec<ComponentHandle> {
        &mut self.tendons
    }
}
