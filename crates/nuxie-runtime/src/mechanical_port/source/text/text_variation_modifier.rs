use crate::mechanical_port::source::{
    generated::text::text_variation_modifier_base::TextVariationModifierBase, text_engine::Font,
};
use std::collections::HashMap;
impl std::ops::Deref for TextVariationModifier {
    type Target = TextVariationModifierBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextVariationModifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextVariationModifier {
    pub const TYPE_KEY: u16 = TextVariationModifierBase::TYPE_KEY;
}

#[derive(Default)]
pub struct TextVariationModifier {
    pub base: TextVariationModifierBase,
}
impl TextVariationModifier {
    pub fn modify(
        &self,
        font: &dyn Font,
        variations: &mut HashMap<u32, f32>,
        font_size: f32,
        strength: f32,
    ) -> f32 {
        let tag = self.base.axis_tag();
        let from = variations
            .get(&tag)
            .copied()
            .unwrap_or_else(|| font.get_axis_value(tag));
        variations.insert(
            tag,
            from * (1.0 - strength) + self.base.axis_value() * strength,
        );
        font_size
    }
    pub fn axis_value_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(group) = parent.as_text_modifier_group_mut() {
                    group.shape_modifier_changed();
                }
            });
        }
    }
}

impl super::text_shape_modifier::TextShapeModifierBehavior for TextVariationModifier {
    fn modify(
        &self,
        font: &dyn Font,
        variations: &mut HashMap<u32, f32>,
        font_size: f32,
        strength: f32,
    ) -> f32 {
        TextVariationModifier::modify(self, font, variations, font_size, strength)
    }
}
