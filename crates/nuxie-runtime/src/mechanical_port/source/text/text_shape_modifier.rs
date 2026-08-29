use std::collections::HashMap;

use crate::mechanical_port::source::{
    generated::text::text_shape_modifier_base::TextShapeModifierBase, text_engine::Font,
};

#[derive(Default)]
pub struct TextShapeModifier {
    pub base: TextShapeModifierBase,
}

impl std::ops::Deref for TextShapeModifier {
    type Target = TextShapeModifierBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextShapeModifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub trait TextShapeModifierBehavior {
    fn modify(
        &self,
        font: &dyn Font,
        variations: &mut HashMap<u32, f32>,
        font_size: f32,
        strength: f32,
    ) -> f32;
}
