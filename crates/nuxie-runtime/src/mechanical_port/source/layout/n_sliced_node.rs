use crate::mechanical_port::source::{
    component::{Component, ComponentDirt, ContainerComponent, has_dirt},
    generated::layout::n_sliced_node_base::NSlicedNodeBase,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
        n_slicer_details::{NSlicerDetails, NSlicerDetailsState},
    },
    layout_component::LayoutComponent,
    math::{aabb::Aabb, mat2d::Mat2D, n_slicer_helpers::NSlicerHelpers, vec2d::Vec2D},
    renderer::raw_path::RawPath,
};

pub struct NSlicedNode {
    pub base: NSlicedNodeBase,
    details: NSlicerDetailsState,
    pub map_world_point: Box<dyn Fn(&mut Vec2D)>,
}

impl NSlicedNode {
    pub const TYPE_KEY: u16 = NSlicedNodeBase::TYPE_KEY;
    pub fn new(base: NSlicedNodeBase) -> Self {
        Self {
            base,
            details: NSlicerDetailsState::default(),
            map_world_point: Box::new(|_point| {}),
        }
    }
    fn mark_path_dirty_recursive(&mut self, send_to_layout: bool) {
        self.base.add_dirt_recursive(ComponentDirt::N_SLICER);
        #[cfg(feature = "rive-layout")]
        if send_to_layout {
            let mut parent = self
                .base
                .parent_mut()
                .map(|value| value as *mut ContainerComponent);
            while let Some(pointer) = parent {
                unsafe {
                    if let Some(layout) = (*pointer).as_mut::<LayoutComponent>() {
                        layout.mark_layout_node_dirty(false);
                        break;
                    }
                    parent = (*pointer)
                        .parent_mut()
                        .map(|value| value as *mut ContainerComponent);
                }
            }
        }
    }
    pub fn width_changed(&mut self) {
        self.mark_path_dirty_recursive(true);
    }
    pub fn height_changed(&mut self) {
        self.mark_path_dirty_recursive(true);
    }
    pub fn local_bounds(&self) -> Aabb {
        Aabb::new(
            0.0,
            0.0,
            self.base.initial_width(),
            self.base.initial_height(),
        )
    }
    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        if has_dirt(
            value,
            ComponentDirt::N_SLICER | ComponentDirt::WORLD_TRANSFORM,
        ) {
            self.update_map_world_point();
        }
    }
    fn update_map_world_point(&mut self) {
        let world = *self.base.world_transform();
        let Some(inverse_world) = world.inverted() else {
            self.map_world_point = Box::new(|_point| {});
            return;
        };
        if self.base.initial_height() <= 0.0 || self.base.initial_width() <= 0.0 {
            self.map_world_point = Box::new(|_point| {});
            return;
        }
        let size = Vec2D::new(self.base.initial_width(), self.base.initial_height());
        let scale = self.scale_for_n_slicer();
        let x_px_stops = NSlicerHelpers::px_stops(&self.details.xs, size.x);
        let y_px_stops = NSlicerHelpers::px_stops(&self.details.ys, size.y);
        let x_uv_stops = NSlicerHelpers::uv_stops(&self.details.xs, size.x);
        let y_uv_stops = NSlicerHelpers::uv_stops(&self.details.ys, size.y);
        let x_scale_info = NSlicerHelpers::analyze_uv_stops(&x_uv_stops, size.x, scale.x.abs());
        // Preserve the pinned size.x argument for the Y-axis analysis.
        let y_scale_info = NSlicerHelpers::analyze_uv_stops(&y_uv_stops, size.x, scale.y.abs());
        let this = self as *const NSlicedNode;
        self.map_world_point = Box::new(move |world_point| unsafe {
            let local = inverse_world * *world_point;
            let sliced = Vec2D::new(
                if scale.x == 0.0 {
                    0.0
                } else {
                    NSlicerHelpers::map_value(
                        &x_px_stops,
                        &x_scale_info,
                        (*this).base.width().abs(),
                        local.x,
                    ) * 1.0_f32.copysign(scale.x)
                },
                if scale.y == 0.0 {
                    0.0
                } else {
                    NSlicerHelpers::map_value(
                        &y_px_stops,
                        &y_scale_info,
                        (*this).base.height().abs(),
                        local.y,
                    ) * 1.0_f32.copysign(scale.y)
                },
            );
            *world_point = world * sliced;
        });
    }
    pub fn as_component(&mut self) -> &mut Component {
        self.base.as_component_mut()
    }
    pub fn deform_world_render_path(&self, path: &mut RawPath) {
        NSlicerHelpers::deform_world_render_path_with_n_slicer(self, path);
    }
    pub fn deform_local_render_path(
        &self,
        path: &mut RawPath,
        world: &Mat2D,
        inverse_world: &Mat2D,
    ) {
        NSlicerHelpers::deform_local_render_path_with_n_slicer(self, path, world, inverse_world);
    }
    pub fn deform_local_point(&self, point: Vec2D, world: &Mat2D, inverse_world: &Mat2D) -> Vec2D {
        *inverse_world * self.deform_world_point(*world * point)
    }
    pub fn deform_world_point(&self, point: Vec2D) -> Vec2D {
        let mut result = point;
        (self.map_world_point)(&mut result);
        result
    }
    pub fn scale_for_n_slicer(&self) -> Vec2D {
        Vec2D::new(
            self.base.width() / self.base.initial_width(),
            self.base.height() / self.base.initial_height(),
        )
    }
    pub fn measure_layout(
        &self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        let measured_width = if width_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            width
        };
        let measured_height = if height_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            height
        };
        Vec2D::new(
            cpp_min(measured_width, self.base.width()),
            cpp_min(measured_height, self.base.height()),
        )
    }
    pub fn control_size(
        &mut self,
        size: Vec2D,
        _width_scale_type: LayoutScaleType,
        _height_scale_type: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        self.base.set_width(size.x);
        self.base.set_height(size.y);
        self.base.mark_world_transform_dirty();
        self.mark_path_dirty_recursive(false);
    }
    pub fn should_propagate_size_to_children(&self) -> bool {
        false
    }
}

fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}

impl NSlicerDetails for NSlicedNode {
    fn details_state(&mut self) -> &mut NSlicerDetailsState {
        &mut self.details
    }
    fn axis_changed(&mut self) {
        self.mark_path_dirty_recursive(true);
    }
}
