use crate::mechanical_port::source::{
    component::Component,
    math::mat2d::Mat2D,
    shapes::{
        paint::shape_paint::ShapePaint, path_flags::PathFlags, shape_paint_path::ShapePaintPath,
    },
};

pub struct ShapePaintContainer {
    path_flags: PathFlags,
    shape_paints: Vec<ShapePaint>,
}

impl Default for ShapePaintContainer {
    fn default() -> Self {
        Self {
            path_flags: PathFlags::NONE,
            shape_paints: Vec::new(),
        }
    }
}

impl ShapePaintContainer {
    pub fn from_component(component: &Component) -> Option<&Self> {
        match component.core_type() {
            TYPE_ARTBOARD
            | TYPE_LAYOUT_COMPONENT
            | TYPE_SHAPE
            | TYPE_TEXT_STYLE_PAINT
            | TYPE_FOREGROUND_LAYOUT_DRAWABLE
            | TYPE_TEXT_INPUT_CURSOR
            | TYPE_TEXT_INPUT_SELECTION
            | TYPE_TEXT_INPUT_TEXT
            | TYPE_TEXT_INPUT_SELECTED_TEXT => component.as_shape_paint_container(),
            _ => None,
        }
    }

    pub fn from_component_mut(component: &mut Component) -> Option<&mut Self> {
        match component.core_type() {
            TYPE_ARTBOARD
            | TYPE_LAYOUT_COMPONENT
            | TYPE_SHAPE
            | TYPE_TEXT_STYLE_PAINT
            | TYPE_FOREGROUND_LAYOUT_DRAWABLE
            | TYPE_TEXT_INPUT_CURSOR
            | TYPE_TEXT_INPUT_SELECTION
            | TYPE_TEXT_INPUT_TEXT
            | TYPE_TEXT_INPUT_SELECTED_TEXT => component.as_shape_paint_container_mut(),
            _ => None,
        }
    }

    pub fn add_paint(&mut self, paint: ShapePaint) {
        self.shape_paints.push(paint);
    }

    pub fn path_flags(&self) -> PathFlags {
        self.shape_paints
            .iter()
            .fold(self.path_flags, |flags, paint| flags | paint.path_flags())
    }

    pub fn invalidate_stroke_effects(&mut self) {
        for paint in &mut self.shape_paints {
            paint.invalidate_effects();
        }
    }

    pub fn propagate_opacity(&mut self, opacity: f32) {
        for paint in &mut self.shape_paints {
            paint.set_render_opacity(opacity);
        }
    }

    pub fn shape_paints(&self) -> &[ShapePaint] {
        &self.shape_paints
    }

    pub fn world_path(&mut self) -> Option<&mut ShapePaintPath> {
        None
    }

    pub fn local_path(&mut self) -> Option<&mut ShapePaintPath> {
        None
    }

    pub fn local_clockwise_path(&mut self) -> Option<&mut ShapePaintPath> {
        None
    }

    pub fn shape_world_transform(&self) -> Mat2D {
        Mat2D::identity()
    }
}
