use crate::mechanical_port::source::{
    component::ComponentHandle,
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
    pub fn translation(&mut self) -> &mut Vec2D {
        &mut self.translation
    }

    pub fn on_added_dirty(
        &mut self,
        this: ComponentHandle,
        context: &mut CoreContext,
    ) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.base.parent() else {
            return StatusCode::MissingObject;
        };
        if !context.is_vertex(parent) {
            return StatusCode::MissingObject;
        }
        context
            .vertex_mut(parent)
            .expect("a component classified as Vertex must resolve as Vertex")
            .set_weight(this);
        StatusCode::Ok
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
