use crate::mechanical_port::source::{
    core_context::CoreContext, generated::text::text_modifier_base::TextModifierBase,
    status_code::StatusCode,
};
pub struct TextModifier {
    pub base: TextModifierBase,
}
impl TextModifier {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(group), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        let added = group
            .with_mut(|group| {
                group
                    .as_text_modifier_group_mut()
                    .map(|group| group.add_modifier(this))
            })
            .flatten()
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
}
