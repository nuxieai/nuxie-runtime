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
    pub fn in_values(&self) -> u32 {
        self.base.in_values()
    }

    pub fn set_in_values(&mut self, value: u32) {
        if self.base.set_in_values_value(value) {
            self.base
                .base
                .core_mut()
                .notify_property_changed(CubicWeightBase::IN_VALUES_PROPERTY_KEY);
        }
    }

    pub fn in_indices(&self) -> u32 {
        self.base.in_indices()
    }

    pub fn set_in_indices(&mut self, value: u32) {
        if self.base.set_in_indices_value(value) {
            self.base
                .base
                .core_mut()
                .notify_property_changed(CubicWeightBase::IN_INDICES_PROPERTY_KEY);
        }
    }

    pub fn out_values(&self) -> u32 {
        self.base.out_values()
    }

    pub fn set_out_values(&mut self, value: u32) {
        if self.base.set_out_values_value(value) {
            self.base
                .base
                .core_mut()
                .notify_property_changed(CubicWeightBase::OUT_VALUES_PROPERTY_KEY);
        }
    }

    pub fn out_indices(&self) -> u32 {
        self.base.out_indices()
    }

    pub fn set_out_indices(&mut self, value: u32) {
        if self.base.set_out_indices_value(value) {
            self.base
                .base
                .core_mut()
                .notify_property_changed(CubicWeightBase::OUT_INDICES_PROPERTY_KEY);
        }
    }

    pub fn in_translation(&mut self) -> &mut Vec2D {
        &mut self.in_translation
    }

    pub fn out_translation(&mut self) -> &mut Vec2D {
        &mut self.out_translation
    }
}
