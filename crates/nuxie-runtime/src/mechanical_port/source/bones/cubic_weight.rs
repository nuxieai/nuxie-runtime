use crate::mechanical_port::source::{
    generated::bones::cubic_weight_base::CubicWeightBase, math::vec2d::Vec2D,
};

pub struct CubicWeight {
    pub base: CubicWeightBase,
    in_translation: Vec2D,
    out_translation: Vec2D,
}

impl Default for CubicWeight {
    fn default() -> Self {
        Self {
            base: CubicWeightBase::default(),
            // Valid source lifecycle writes these cached deformation results
            // before reading them; Rust initializes the storage eagerly.
            in_translation: Vec2D::new(0.0, 0.0),
            out_translation: Vec2D::new(0.0, 0.0),
        }
    }
}

impl CubicWeight {
    pub fn in_translation(&mut self) -> &mut Vec2D {
        &mut self.in_translation
    }

    pub fn out_translation(&mut self) -> &mut Vec2D {
        &mut self.out_translation
    }
}
