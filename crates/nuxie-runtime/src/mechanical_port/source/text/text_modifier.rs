use crate::mechanical_port::source::{
    core_context::CoreContext, generated::text::text_modifier_base::TextModifierBase,
    status_code::StatusCode,
};
pub struct TextModifier {
    pub base: TextModifierBase,
}
impl TextModifier {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(mut group) = self.base.parent_as_text_modifier_group() else {
            return StatusCode::MissingObject;
        };
        unsafe { group.as_mut() }.add_modifier(self);
        StatusCode::Ok
    }
}
