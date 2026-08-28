use crate::mechanical_port::source::{
    generated::text::text_variation_modifier_base::TextVariationModifierBase, text_engine::Font,
};
use std::collections::HashMap;
pub struct TextVariationModifier {
    pub base: TextVariationModifierBase,
}
impl TextVariationModifier {
    pub fn modify(
        &self,
        font: &Font,
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
