use crate::mechanical_port::source::{
    core_context::CoreContext, generated::text::text_style_feature_base::TextStyleFeatureBase,
    status_code::StatusCode,
};

impl std::ops::Deref for TextStyleFeature {
    type Target = TextStyleFeatureBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleFeature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextStyleFeature {
    pub const TYPE_KEY: u16 = TextStyleFeatureBase::TYPE_KEY;
}

#[derive(Default)]
pub struct TextStyleFeature {
    pub base: TextStyleFeatureBase,
}

impl TextStyleFeature {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code == StatusCode::Ok {
            let (Some(style), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
                return StatusCode::InvalidObject;
            };
            let added = style
                .with_mut(|style| {
                    style
                        .as_text_style_mut()
                        .map(|style| style.add_feature(this))
                })
                .flatten()
                .is_some();
            if !added {
                return StatusCode::InvalidObject;
            }
        }
        code
    }
}
