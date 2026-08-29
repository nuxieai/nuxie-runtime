use crate::mechanical_port::source::{
    core::CoreHandle, core_context::CoreContext, generated::bones::tendon_base::TendonBase,
    math::mat2d::Mat2D, status_code::StatusCode,
};

pub struct Tendon {
    pub base: TendonBase,
    inverse_bind: Mat2D,
    bone: Option<CoreHandle>,
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
    pub fn bone_id(&self) -> u32 {
        self.base.bone_id()
    }

    pub fn set_bone_id(&mut self, value: u32) {
        if self.base.set_bone_id_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(TendonBase::BONE_ID_PROPERTY_KEY);
        }
    }

    pub fn xx(&self) -> f32 {
        self.base.xx()
    }

    pub fn set_xx(&mut self, value: f32) {
        if self.base.set_xx_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(TendonBase::XX_PROPERTY_KEY);
        }
    }

    pub fn yx(&self) -> f32 {
        self.base.yx()
    }

    pub fn set_yx(&mut self, value: f32) {
        if self.base.set_yx_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(TendonBase::YX_PROPERTY_KEY);
        }
    }

    pub fn xy(&self) -> f32 {
        self.base.xy()
    }

    pub fn set_xy(&mut self, value: f32) {
        if self.base.set_xy_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(TendonBase::XY_PROPERTY_KEY);
        }
    }

    pub fn yy(&self) -> f32 {
        self.base.yy()
    }

    pub fn set_yy(&mut self, value: f32) {
        if self.base.set_yy_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(TendonBase::YY_PROPERTY_KEY);
        }
    }

    pub fn tx(&self) -> f32 {
        self.base.tx()
    }

    pub fn set_tx(&mut self, value: f32) {
        if self.base.set_tx_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(TendonBase::TX_PROPERTY_KEY);
        }
    }

    pub fn ty(&self) -> f32 {
        self.base.ty()
    }

    pub fn set_ty(&mut self, value: f32) {
        if self.base.set_ty_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(TendonBase::TY_PROPERTY_KEY);
        }
    }

    pub fn bone(&self) -> Option<CoreHandle> {
        self.bone.clone()
    }

    pub fn inverse_bind(&self) -> &Mat2D {
        &self.inverse_bind
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
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

        let code = self.base.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(core_object) = context.resolve(self.base.bone_id()) else {
            return StatusCode::MissingObject;
        };
        if !core_object.is_type_of(
            crate::mechanical_port::source::generated::bones::bone_base::BoneBase::TYPE_KEY,
        ) {
            return StatusCode::MissingObject;
        }
        self.bone = Some(core_object);
        StatusCode::Ok
    }

    pub fn on_added_clean(
        &mut self,
        this: CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        let Some(parent) = context.resolve(self.base.base.base.parent_id()) else {
            return StatusCode::MissingObject;
        };
        if !parent.is_type_of(
            crate::mechanical_port::source::generated::bones::skin_base::SkinBase::TYPE_KEY,
        ) {
            return StatusCode::MissingObject;
        }
        parent
            .with_downcast_mut::<crate::mechanical_port::source::bones::skin::Skin, _>(|skin| {
                skin.add_tendon(this)
            })
            .map_or(StatusCode::MissingObject, |_| StatusCode::Ok)
    }
}
