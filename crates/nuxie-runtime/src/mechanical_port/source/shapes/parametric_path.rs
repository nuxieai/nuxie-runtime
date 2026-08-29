use crate::mechanical_port::source::{
    generated::shapes::parametric_path_base::ParametricPathBase,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    math::{aabb::Aabb, vec2d::Vec2D},
};
impl std::ops::Deref for ParametricPath {
    type Target = ParametricPathBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ParametricPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ParametricPath {
    pub const TYPE_KEY: u16 = ParametricPathBase::TYPE_KEY;
}

#[derive(Default)]
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
        self.set_width(size.x);
        self.set_height(size.y);
        self.base.mark_world_transform_dirty();
        self.mark_path_dirty(false);
    }
    pub fn mark_path_dirty(&mut self, send_to_layout: bool) {
        self.base.base.mark_path_dirty(true);
        if send_to_layout {
            let shape = self.base.shape_handle();
            let mut parent = self.base.parent_handle();
            while let Some(current) = parent {
                if current.is_type_of(
                    crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
                ) {
                    crate::mechanical_port::source::layout_component::LayoutComponent::mark_layout_node_dirty_occurrence(&current, false);
                    break;
                }
                let is_node = current.is_type_of(
                    crate::mechanical_port::source::generated::node_base::NodeBase::TYPE_KEY,
                );
                if is_node {
                    if current
                        .is_type_of(crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY)
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
    pub fn set_width(&mut self, value: f32) {
        if self.base.set_width_value(value) {
            self.width_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                ParametricPathBase::WIDTH_PROPERTY_KEY,
            );
        }
    }
    pub fn set_height(&mut self, value: f32) {
        if self.base.set_height_value(value) {
            self.height_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                ParametricPathBase::HEIGHT_PROPERTY_KEY,
            );
        }
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
