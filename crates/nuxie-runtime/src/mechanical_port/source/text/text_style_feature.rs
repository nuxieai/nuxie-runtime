use crate::mechanical_port::source::{
    core_context::CoreContext, generated::text::text_style_feature_base::TextStyleFeatureBase,
    status_code::StatusCode,
};

pub struct TextStyleFeature {
    pub base: TextStyleFeatureBase,
}

impl TextStyleFeature {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code == StatusCode::Ok {
            let Some(mut style) = self.base.parent_as_text_style() else {
                return StatusCode::InvalidObject;
            };
            unsafe { style.as_mut() }.add_feature(self);
        }
        code
    }
}
