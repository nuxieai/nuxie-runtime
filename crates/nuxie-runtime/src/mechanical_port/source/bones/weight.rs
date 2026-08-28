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
        let Some(parent) = self.base.parent() else {
            return StatusCode::MissingObject;
        };
        parent
            .with_mut(|parent| {
                let Some(vertex) = parent.as_vertex_mut() else {
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
            xx += bone_transforms[transform_index] * normalized_weight;
            transform_index += 1;
            xy += bone_transforms[transform_index] * normalized_weight;
            transform_index += 1;
            yx += bone_transforms[transform_index] * normalized_weight;
            transform_index += 1;
            yy += bone_transforms[transform_index] * normalized_weight;
            transform_index += 1;
            tx += bone_transforms[transform_index] * normalized_weight;
            transform_index += 1;
            ty += bone_transforms[transform_index] * normalized_weight;
        }

        Mat2D::new(xx, xy, yx, yy, tx, ty).transform_point(world.transform_point(in_point))
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
