use crate::mechanical_port::source::{
    core::CoreHandle, core_context::CoreContext,
    generated::text::text_target_modifier_base::TextTargetModifierBase, status_code::StatusCode,
};

impl std::ops::Deref for TextTargetModifier {
    type Target = TextTargetModifierBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextTargetModifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextTargetModifier {
    pub const TYPE_KEY: u16 = TextTargetModifierBase::TYPE_KEY;
}

#[derive(Default)]
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
                .is_type_of(crate::mechanical_port::source::generated::transform_component_base::TransformComponentBase::TYPE_KEY)
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
