use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    core::{CoreHandle, CoreObject},
    shapes::{paint::paint_cells::PaintCells, path_flags::PathFlags},
};

pub struct ShapePaintContainer {
    // Replaced, never edited, when a paint is added.
    shape_paints: Rc<[CoreHandle]>,
    cells: Rc<ContainerCells>,
}

#[cfg(test)]
mod registration_tests {
    use super::*;
    use crate::mechanical_port::source::{core::CoreArena, shapes::paint::fill::Fill};

    #[test]
    fn paint_registration_matches_editor_identity_deduplication() {
        let arena = CoreArena::default();
        let first = arena.insert(Fill::default());
        let second = arena.insert(Fill::default());
        let mut container = ShapePaintContainer::default();
        for paint in [&first, &first, &second] {
            let cells = paint
                .with(|paint| Rc::clone(paint.as_shape_paint().unwrap().cells()))
                .unwrap();
            container.add_paint(paint.clone(), cells);
        }
        #[cfg(feature = "editor")]
        assert_eq!(container.shape_paints(), &[first, second]);
        #[cfg(not(feature = "editor"))]
        assert_eq!(container.shape_paints(), &[first.clone(), first, second]);
        assert_eq!(
            container.cells.paints.borrow().len(),
            container.shape_paints().len()
        );
    }
}

/// A paint container's path flags and paints, shared with the path pipeline's
/// cells (ShapeCells) so they read them without borrowing the container.
pub(crate) struct ContainerCells {
    // The container's own flags (clipping, followPath, neverDeferUpdate).
    flags: Cell<PathFlags>,
    // Replaced, never edited, when a paint is added, so a pathChanged pass can
    // hold it across paint callbacks.
    paints: RefCell<Rc<[ContainerPaint]>>,
    // The union of the paints' flags, refreshed when a paint is added or a
    // paint's flags change.
    paint_flags: Cell<PathFlags>,
}

pub(crate) struct ContainerPaint {
    pub(crate) handle: CoreHandle,
    pub(crate) cells: Rc<PaintCells>,
}

impl Default for ContainerCells {
    fn default() -> Self {
        Self {
            flags: Cell::new(PathFlags::NONE),
            paints: RefCell::new(Rc::new([])),
            paint_flags: Cell::new(PathFlags::NONE),
        }
    }
}

impl ContainerCells {
    /// ShapePaintContainer::pathFlags(): the container's own flags and every
    /// paint's.
    #[inline]
    pub(crate) fn path_flags(&self) -> PathFlags {
        self.flags.get() | self.paint_flags.get()
    }

    /// Recompute the paints' flag union after a paint's flags change.
    pub(crate) fn refresh_paint_flags(&self) {
        let flags = self
            .paints
            .borrow()
            .iter()
            .fold(PathFlags::NONE, |flags, paint| flags | paint.cells.path_flags());
        self.paint_flags.set(flags);
    }

    pub(crate) fn paints(&self) -> Rc<[ContainerPaint]> {
        Rc::clone(&self.paints.borrow())
    }

    #[inline]
    pub(crate) fn is_flagged(&self, flags: PathFlags) -> bool {
        !(self.path_flags() & flags).is_empty()
    }

    /// ShapePaintContainer::invalidateStrokeEffects().
    pub(crate) fn invalidate_stroke_effects(&self) {
        let paints = Rc::clone(&self.paints.borrow());
        for paint in paints.iter() {
            paint.cells.invalidate_effects(&paint.handle);
        }
    }
}

impl Default for ShapePaintContainer {
    fn default() -> Self {
        Self {
            shape_paints: Rc::new([]),
            cells: Rc::default(),
        }
    }
}

impl ShapePaintContainer {
    pub fn add_path_flags(&mut self, flags: PathFlags) {
        self.cells.flags.set(self.cells.flags.get() | flags);
    }

    pub fn from_component(component: &dyn CoreObject) -> Option<&Self> {
        component.as_shape_paint_container()
    }

    pub fn from_component_mut(component: &mut dyn CoreObject) -> Option<&mut Self> {
        component.as_shape_paint_container_mut()
    }

    pub(crate) fn cells(&self) -> &Rc<ContainerCells> {
        &self.cells
    }

    pub(crate) fn add_paint(&mut self, paint: CoreHandle, cells: Rc<PaintCells>) {
        #[cfg(feature = "editor")]
        if self.shape_paints.contains(&paint) {
            return;
        }
        cells.set_container(Rc::downgrade(&self.cells));
        self.shape_paints = self
            .shape_paints
            .iter()
            .cloned()
            .chain(std::iter::once(paint.clone()))
            .collect();
        let paints: Rc<[ContainerPaint]> = self
            .cells
            .paints
            .borrow()
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
        *self.cells.paints.borrow_mut() = paints;
        self.cells.refresh_paint_flags();
    }

    pub fn path_flags(&self) -> PathFlags {
        self.cells.path_flags()
    }

    pub fn invalidate_stroke_effects(&mut self) {
        self.cells.invalidate_stroke_effects();
    }

    pub(crate) fn invalidate_stroke_effects_occurrence(owner: &CoreHandle) {
        let cells = owner
            .with(|object| {
                object
                    .as_shape_paint_container()
                    .map(|container| Rc::clone(&container.cells))
            })
            .flatten();
        if let Some(cells) = cells {
            cells.invalidate_stroke_effects();
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

    /// The paints, as a list that outlives a borrow of this container.
    pub fn shape_paint_list(&self) -> Rc<[CoreHandle]> {
        Rc::clone(&self.shape_paints)
    }
}
