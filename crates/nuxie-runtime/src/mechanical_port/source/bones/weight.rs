use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::CoreContext,
    generated::bones::weight_base::WeightBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    status_code::StatusCode,
};

pub struct Weight {
    pub base: WeightBase,
    translation: Vec2D,
}

impl Default for Weight {
    fn default() -> Self {
        Self {
            base: WeightBase::default(),
            // Valid source lifecycle writes this deformation cache before it
            // is read; Rust initializes the storage eagerly.
            translation: Vec2D::new(0.0, 0.0),
        }
    }
}

impl Weight {
    pub(crate) fn core_mut(&mut self) -> &mut crate::mechanical_port::source::core::Core {
        &mut self.base.base.base.base
    }

    pub fn values(&self) -> u32 {
        self.base.values()
    }

    pub fn set_values(&mut self, value: u32) {
        if self.base.set_values_value(value) {
            self.core_mut()
                .notify_property_changed(WeightBase::VALUES_PROPERTY_KEY);
        }
    }

    pub fn indices(&self) -> u32 {
        self.base.indices()
    }

    pub fn set_indices(&mut self, value: u32) {
        if self.base.set_indices_value(value) {
            self.core_mut()
                .notify_property_changed(WeightBase::INDICES_PROPERTY_KEY);
        }
    }

    pub fn translation(&mut self) -> &mut Vec2D {
        &mut self.translation
    }

    pub fn on_added_dirty(
        &mut self,
        this: CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.base.parent_handle() else {
            return StatusCode::MissingObject;
        };
        parent
            .with_mut(|parent| {
                let Some(vertex) = parent.as_vertex_behavior_mut() else {
                    return StatusCode::MissingObject;
                };
                vertex.set_weight(this);
                StatusCode::Ok
            })
            .unwrap_or(StatusCode::MissingObject)
    }

    fn encoded_weight_value(index: u32, data: u32) -> usize {
        ((data >> (index * 8)) & 0xff) as usize
    }

    pub fn deform(
        in_point: Vec2D,
        indices: u32,
        weights: u32,
        world: &Mat2D,
        bone_transforms: &[f32],
    ) -> Vec2D {
        let (mut xx, mut xy, mut yx, mut yy, mut tx, mut ty) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for index_in_packed_values in 0..4 {
            let weight = Self::encoded_weight_value(index_in_packed_values, weights);
            if weight == 0 {
                continue;
            }

            let normalized_weight = weight as f32 / 255.0;
            let bone_index = Self::encoded_weight_value(index_in_packed_values, indices);
            let mut transform_index = bone_index * 6;
            xx = bone_transforms[transform_index].mul_add(normalized_weight, xx);
            transform_index += 1;
            xy = bone_transforms[transform_index].mul_add(normalized_weight, xy);
            transform_index += 1;
            yx = bone_transforms[transform_index].mul_add(normalized_weight, yx);
            transform_index += 1;
            yy = bone_transforms[transform_index].mul_add(normalized_weight, yy);
            transform_index += 1;
            tx = bone_transforms[transform_index].mul_add(normalized_weight, tx);
            transform_index += 1;
            ty = bone_transforms[transform_index].mul_add(normalized_weight, ty);
        }

        Mat2D::new(xx, xy, yx, yy, tx, ty) * (*world * in_point)
    }
}
impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks for Weight {
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
impl crate::mechanical_port::source::generated::bones::weight_base::WeightBaseCallbacks for Weight {
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deform_matches_pinned_fused_weight_accumulation() {
        let bone_transforms = [
            1.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.797685921,
            -0.00870515127,
            0.00870512333,
            0.797685862,
            1.2355957,
            100.055695,
            -0.187692404,
            -0.775338828,
            0.771069884,
            -0.205327198,
            284.760468,
            272.738983,
            -0.246825233,
            0.974436581,
            -0.969326734,
            -0.224662051,
            300.711243,
            -294.504639,
        ];
        let world = Mat2D::new(
            0.835797548,
            -0.167389423,
            0.159021586,
            0.884335815,
            352.651398,
            220.692368,
        );
        let indices = 1 | (2 << 8) | (3 << 16);
        let weights = 100 | (100 << 8) | (55 << 16);

        let result = Weight::deform(
            Vec2D::new(12.345678, -27.3720322),
            indices,
            weights,
            &world,
            &bone_transforms,
        );
        assert_eq!(result.x.to_bits(), 0x4383_41c7);
        assert_eq!(result.y.to_bits(), 0x42a7_0c34);
    }
}
