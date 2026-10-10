use std::rc::Rc;

use crate::mechanical_port::source::{
    core::{CoreHandle, CoreObject},
    shapes::{paint::paint_cells::PaintCells, path_flags::PathFlags},
};

pub struct ShapePaintContainer {
    path_flags: PathFlags,
    shape_paints: Vec<CoreHandle>,
    // The same paints with their cells. Replaced, never edited, when a paint
    // is added, so a pathChanged pass can hold it across paint callbacks.
    paints: Rc<[ContainerPaint]>,
}

struct ContainerPaint {
    handle: CoreHandle,
    cells: Rc<PaintCells>,
}

impl Default for ShapePaintContainer {
    fn default() -> Self {
        Self {
            path_flags: PathFlags::NONE,
            shape_paints: Vec::new(),
            paints: Rc::new([]),
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

    pub(crate) fn add_paint(&mut self, paint: CoreHandle, cells: Rc<PaintCells>) {
        self.shape_paints.push(paint.clone());
        self.paints = self
            .paints
            .iter()
            .map(|entry| ContainerPaint {
                handle: entry.handle.clone(),
                cells: Rc::clone(&entry.cells),
            })
            .chain(std::iter::once(ContainerPaint {
                handle: paint,
                cells,
            }))
            .collect();
    }

    /// ShapePaintContainer::pathFlags(): the container's own flags and every
    /// paint's.
    pub fn path_flags(&self) -> PathFlags {
        self.paints
            .iter()
            .fold(self.path_flags, |flags, paint| flags | paint.cells.path_flags())
    }

    pub fn invalidate_stroke_effects(&mut self) {
        Self::invalidate_paint_effects(&self.paints);
    }

    pub(crate) fn invalidate_stroke_effects_occurrence(owner: &CoreHandle) {
        let paints = owner
            .with(|object| {
                object
                    .as_shape_paint_container()
                    .map(|container| Rc::clone(&container.paints))
            })
            .flatten();
        if let Some(paints) = paints {
            Self::invalidate_paint_effects(&paints);
        }
    }

    fn invalidate_paint_effects(paints: &[ContainerPaint]) {
        for paint in paints {
            paint.cells.invalidate_effects(&paint.handle);
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
