use crate::mechanical_port::source::{
    component::Component,
    core::CoreHandle,
    math::mat2d::Mat2D,
    shapes::{
        paint::shape_paint::ShapePaint, path_flags::PathFlags, shape_paint_path::ShapePaintPath,
    },
};

pub struct ShapePaintContainer {
    path_flags: PathFlags,
    shape_paints: Vec<CoreHandle>,
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
    pub fn add_path_flags(&mut self, flags: PathFlags) {
        self.path_flags |= flags;
    }

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

    pub fn add_paint(&mut self, paint: CoreHandle) {
        self.shape_paints.push(paint);
    }

    pub fn path_flags(&self) -> PathFlags {
        self.shape_paints
            .iter()
            .fold(self.path_flags, |flags, paint| {
                flags
                    | paint
                        .with(|paint| paint.as_shape_paint().map(ShapePaint::path_flags))
                        .flatten()
                        .unwrap_or(PathFlags::NONE)
            })
    }

    pub fn invalidate_stroke_effects(&mut self) {
        for paint in self.shape_paints.iter().cloned() {
            paint.with_mut(|paint| {
                if let Some(paint) = paint.as_shape_paint_mut() {
                    paint.invalidate_effects();
                }
            });
        }
    }

    pub fn propagate_opacity(&mut self, opacity: f32) {
        for paint in self.shape_paints.iter().cloned() {
            paint.with_mut(|paint| {
                if let Some(paint) = paint.as_shape_paint_mut() {
                    paint.set_render_opacity(opacity);
                }
            });
        }
    }

    pub fn shape_paints(&self) -> &[CoreHandle] {
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
