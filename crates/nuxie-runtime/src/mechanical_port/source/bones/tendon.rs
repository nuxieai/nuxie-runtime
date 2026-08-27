use crate::mechanical_port::source::{
    component::ComponentHandle, core_context::CoreContext,
    generated::bones::tendon_base::TendonBase, math::mat2d::Mat2D, status_code::StatusCode,
};

pub struct Tendon {
    pub base: TendonBase,
    inverse_bind: Mat2D,
    bone: Option<ComponentHandle>,
}

impl Default for Tendon {
    fn default() -> Self {
        Self {
            base: TendonBase::default(),
            inverse_bind: Mat2D::identity(),
            bone: None,
        }
    }
}

impl Tendon {
    pub fn bone(&self) -> Option<ComponentHandle> {
        self.bone
    }

    pub fn inverse_bind(&self) -> &Mat2D {
        &self.inverse_bind
    }

    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let bind = Mat2D::new(
            self.base.xx(),
            self.base.xy(),
            self.base.yx(),
            self.base.yy(),
            self.base.tx(),
            self.base.ty(),
        );

        // Failed inversion leaves the identity destination unchanged.
        self.inverse_bind = bind.invert_or_identity();

        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(core_object) = context.resolve(self.base.bone_id()) else {
            return StatusCode::MissingObject;
        };
        if !context.is_bone(core_object) {
            return StatusCode::MissingObject;
        }
        self.bone = Some(core_object);
        StatusCode::Ok
    }

    pub fn on_added_clean(
        &mut self,
        this: ComponentHandle,
        context: &mut CoreContext,
    ) -> StatusCode {
        let Some(parent) = self.base.parent() else {
            return StatusCode::MissingObject;
        };
        if !context.is_skin(parent) {
            return StatusCode::MissingObject;
        }
        context
            .skin_mut(parent)
            .expect("a component classified as Skin must resolve as Skin")
            .add_tendon(this);
        StatusCode::Ok
    }
}
