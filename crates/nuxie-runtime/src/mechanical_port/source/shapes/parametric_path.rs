use crate::mechanical_port::source::{
    generated::shapes::parametric_path_base::ParametricPathBase,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    math::{aabb::Aabb, vec2d::Vec2D},
};
pub struct ParametricPath {
    pub base: ParametricPathBase,
}
impl ParametricPath {
    pub fn measure_layout(
        &self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Vec2D::new(
            if width_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                width
            }
            .min(self.base.width()),
            if height_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                height
            }
            .min(self.base.height()),
        )
    }
    pub fn control_size(
        &mut self,
        size: Vec2D,
        _width: LayoutScaleType,
        _height: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        self.base.set_width(size.x);
        self.base.set_height(size.y);
        self.base.mark_world_transform_dirty();
        self.mark_path_dirty(false);
    }
    pub fn mark_path_dirty(&mut self, send_to_layout: bool) {
        self.base.super_mark_path_dirty();
        if send_to_layout {
            let shape = self.base.shape_handle();
            let mut parent = self.base.parent_handle();
            while let Some(current) = parent {
                let found_layout = current
                    .with_mut(|current| {
                        if let Some(layout) = current.as_layout_component_mut() {
                            layout.mark_layout_node_dirty(false);
                            true
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);
                if found_layout {
                    break;
                }
                let is_node = current
                    .with(|current| current.as_node().is_some())
                    .unwrap_or(false);
                if is_node {
                    if current
                        .with(|current| current.as_shape().is_some())
                        .unwrap_or(false)
                        && shape.as_ref() == Some(&current)
                    {
                        parent = current
                            .with(|current| current.component_parent_handle())
                            .flatten();
                        continue;
                    }
                    break;
                }
                parent = current
                    .with(|current| current.component_parent_handle())
                    .flatten();
            }
        }
    }
    pub fn try_property_bounds(&self, result: &mut Aabb) -> bool {
        *result = Aabb::from_ltwh(
            -self.base.origin_x() * self.base.width(),
            -self.base.origin_y() * self.base.height(),
            self.base.width(),
            self.base.height(),
        );
        true
    }
    pub fn width_changed(&mut self) {
        self.mark_path_dirty(true);
    }
    pub fn height_changed(&mut self) {
        self.mark_path_dirty(true);
    }
    pub fn origin_x_changed(&mut self) {
        self.mark_path_dirty(true);
    }
    pub fn origin_y_changed(&mut self) {
        self.mark_path_dirty(true);
    }
}
