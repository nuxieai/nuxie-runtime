use crate::mechanical_port::source::{
    core::CoreHandle, core_context::CoreContext,
    generated::text::text_target_modifier_base::TextTargetModifierBase, status_code::StatusCode,
};

pub struct TextTargetModifier {
    pub base: TextTargetModifierBase,
    target: Option<CoreHandle>,
}
impl TextTargetModifier {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.target = context.resolve(self.base.target_id()).filter(|target| {
            target
                .with(|target| target.as_transform_component().is_some())
                .unwrap_or(false)
        });
        StatusCode::Ok
    }

    pub fn target(&self) -> Option<CoreHandle> {
        self.target.clone()
    }

    pub fn text_component(&self) -> Option<CoreHandle> {
        self.base.parent_handle().and_then(|group| {
            group.with(|group| {
                group
                    .as_text_modifier_group()
                    .and_then(|group| group.text_component())
            })?
        })
    }
}
