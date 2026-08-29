use crate::mechanical_port::source::{
    core_context::CoreContext, generated::text::text_modifier_base::TextModifierBase,
    status_code::StatusCode,
};
impl std::ops::Deref for TextModifier {
    type Target = TextModifierBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextModifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextModifier {
    pub const TYPE_KEY: u16 = TextModifierBase::TYPE_KEY;
}

#[derive(Default)]
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
