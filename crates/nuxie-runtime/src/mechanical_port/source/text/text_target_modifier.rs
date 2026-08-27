use super::text::Text;
use crate::mechanical_port::source::{
    core_context::CoreContext, generated::text::text_target_modifier_base::TextTargetModifierBase,
    status_code::StatusCode, transform_component::TransformComponent,
};
use std::ptr::NonNull;

pub struct TextTargetModifier {
    pub base: TextTargetModifierBase,
    target: Option<NonNull<TransformComponent>>,
}
impl TextTargetModifier {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.target = context.resolve(self.base.target_id()).map(NonNull::cast);
        StatusCode::Ok
    }
    pub fn text_component(&self) -> Option<NonNull<Text>> {
        self.base
            .parent_as_text_modifier_group()
            .and_then(|group| unsafe { group.as_ref() }.text_component())
    }
}
