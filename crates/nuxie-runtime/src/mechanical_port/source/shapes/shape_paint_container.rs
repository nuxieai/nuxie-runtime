use crate::mechanical_port::source::{
    core::{CoreHandle, CoreObject},
    shapes::path_flags::PathFlags,
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

    pub fn from_component(component: &dyn CoreObject) -> Option<&Self> {
        component.as_shape_paint_container()
    }

    pub fn from_component_mut(component: &mut dyn CoreObject) -> Option<&mut Self> {
        component.as_shape_paint_container_mut()
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
                        .with(|paint| {
                            paint
                                .as_shape_paint_behavior()
                                .map(|paint| paint.path_flags())
                        })
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
}
