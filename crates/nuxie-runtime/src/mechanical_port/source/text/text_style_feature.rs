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
    pub fn parent_id_changed(&mut self) {
        if self.base.reparent_css_style_child() {
            if let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) {
                parent.with_mut(|o| {
                    if let Some(style) = o.as_text_style_mut() {
                        style.add_feature(this);
                    }
                });
            }
        }
    }

    pub fn tag_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|o| {
                if let Some(style) = o.as_text_style_mut() {
                    style.css_options_changed();
                }
            });
        }
    }
    pub fn feature_value_changed(&mut self) {
        self.tag_changed();
    }

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
