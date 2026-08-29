use crate::mechanical_port::source::{
    component_dirt::ComponentDirt, core_context::CoreContext,
    generated::text::text_style_axis_base::TextStyleAxisBase, status_code::StatusCode,
};

impl std::ops::Deref for TextStyleAxis {
    type Target = TextStyleAxisBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleAxis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextStyleAxis {
    pub const TYPE_KEY: u16 = TextStyleAxisBase::TYPE_KEY;
}

#[derive(Default)]
pub struct TextStyleAxis {
    pub base: TextStyleAxisBase,
}

impl TextStyleAxis {
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
                        .map(|style| style.add_variation(this))
                })
                .flatten()
                .is_some();
            if !added {
                return StatusCode::InvalidObject;
            }
        }
        code
    }
    pub fn tag_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_component_mut() {
                    parent.add_dirt(ComponentDirt::TEXT_SHAPE, false);
                }
            });
        }
    }
    pub fn axis_value_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_component_mut() {
                    parent.add_dirt(ComponentDirt::TEXT_SHAPE, false);
                }
            });
        }
    }
}
