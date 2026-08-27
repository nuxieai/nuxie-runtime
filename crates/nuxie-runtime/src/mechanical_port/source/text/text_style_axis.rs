use crate::mechanical_port::source::{
    component_dirt::ComponentDirt, core_context::CoreContext,
    generated::text::text_style_axis_base::TextStyleAxisBase, status_code::StatusCode,
};

pub struct TextStyleAxis {
    pub base: TextStyleAxisBase,
}

impl TextStyleAxis {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code == StatusCode::Ok {
            let Some(mut style) = self.base.parent_as_text_style() else {
                return StatusCode::InvalidObject;
            };
            unsafe { style.as_mut() }.add_variation(self);
        }
        code
    }
    pub fn tag_changed(&mut self) {
        self.base.parent_mut().add_dirt(ComponentDirt::TEXT_SHAPE);
    }
    pub fn axis_value_changed(&mut self) {
        self.base.parent_mut().add_dirt(ComponentDirt::TEXT_SHAPE);
    }
}
