use std::collections::HashMap;

use crate::mechanical_port::source::text_engine::Font;

pub trait TextShapeModifier {
    fn modify(
        &self,
        font: &Font,
        variations: &mut HashMap<u32, f32>,
        font_size: f32,
        strength: f32,
    ) -> f32;
}
