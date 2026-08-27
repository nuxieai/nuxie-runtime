use crate::mechanical_port::source::{
    component::{Component, ContainerComponent},
    generated::shapes::parametric_path_base::ParametricPathBase,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    layout_component::LayoutComponent,
    math::{aabb::Aabb, vec2d::Vec2D},
    node::Node,
    shapes::shape::Shape,
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
        #[cfg(feature = "rive-layout")]
        if send_to_layout {
            let mut parent = self.base.parent_mut().map(|p| p as *mut ContainerComponent);
            while let Some(pointer) = parent {
                unsafe {
                    if let Some(layout) = (*pointer).as_mut::<LayoutComponent>() {
                        layout.mark_layout_node_dirty(false);
                        break;
                    }
                    if (*pointer).is::<Node>() {
                        if let Some(shape) = (*pointer).as_mut::<Shape>() {
                            if std::ptr::eq(shape, self.base.shape_mut()) {
                                parent = (*pointer).parent_mut().map(|p| p as *mut _);
                                continue;
                            }
                        }
                        break;
                    }
                    parent = (*pointer).parent_mut().map(|p| p as *mut _);
                }
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
