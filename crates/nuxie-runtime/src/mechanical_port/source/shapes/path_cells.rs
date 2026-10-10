//! The state of a Path that the dirt cascade and its update share: its shape
//! link, the once-per-update shape notification, its deferred rebuild and its
//! runtime flags. Held outside the Path so Path::onDirty and markPathDirty run
//! without borrowing it.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    component::{ComponentOccurrenceHandle, SharedDirt, has_dirt},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    shapes::{path_flags::PathFlags, shape_cells::ShapeCells},
};

pub(crate) struct PathCells {
    // The path's own Component dirt.
    dirt: Rc<SharedDirt>,
    shape: RefCell<Option<PathShape>>,
    // Whether this update cycle already told the shape its path changed.
    shape_notified: Cell<bool>,
    // The last update skipped the rebuild while the shape was transparent.
    deferred: Cell<bool>,
    // Runtime flags (followPath, clipping), separate from the authored ones.
    flags: Cell<PathFlags>,
}

#[derive(Clone)]
pub(crate) struct PathShape {
    pub(crate) handle: CoreHandle,
    pub(crate) cells: Rc<ShapeCells>,
}

impl PathCells {
    pub(crate) fn new(dirt: Rc<SharedDirt>) -> Self {
        Self {
            dirt,
            shape: RefCell::new(None),
            shape_notified: Cell::new(false),
            deferred: Cell::new(false),
            flags: Cell::new(PathFlags::NONE),
        }
    }

    pub(crate) fn shape(&self) -> Option<PathShape> {
        self.shape.borrow().clone()
    }

    pub(crate) fn shape_handle(&self) -> Option<CoreHandle> {
        self.shape.borrow().as_ref().map(|shape| shape.handle.clone())
    }

    pub(crate) fn bind_shape(&self, handle: CoreHandle, cells: Rc<ShapeCells>) {
        *self.shape.borrow_mut() = Some(PathShape { handle, cells });
    }

    pub(crate) fn is_flagged(&self, flags: PathFlags) -> bool {
        !(self.flags.get() & flags).is_empty()
    }

    pub(crate) fn add_flags(&self, flags: PathFlags) {
        self.flags.set(self.flags.get() | flags);
    }

    pub(crate) fn deferred(&self) -> bool {
        self.deferred.get()
    }

    pub(crate) fn set_deferred(&self, deferred: bool) {
        self.deferred.set(deferred);
    }

    pub(crate) fn reset_shape_notification(&self) {
        self.shape_notified.set(false);
    }

    /// The shape to notify, once per update cycle.
    pub(crate) fn take_shape_notification(&self) -> Option<PathShape> {
        if self.shape_notified.get() {
            return None;
        }
        let shape = self.shape()?;
        self.shape_notified.set(true);
        Some(shape)
    }

    /// Path::canDeferPathUpdate().
    pub(crate) fn can_defer_path_update(&self) -> bool {
        self.shape.borrow().as_ref().is_some_and(|shape| {
            if !shape.handle.is_occupied() || shape.cells.render_opacity() != 0.0 {
                return false;
            }
            // Shape::canDeferPathUpdate's flags and its followPath check read
            // the same aggregate; fold the paints once.
            let flags = shape.cells.path_flags();
            (flags & (PathFlags::CLIPPING | PathFlags::NEVER_DEFER_UPDATE | PathFlags::FOLLOW_PATH))
                .is_empty()
                && !self.is_flagged(PathFlags::FOLLOW_PATH | PathFlags::CLIPPING)
        })
    }

    /// Path::onDirty for a path no caller has borrowed.
    pub(crate) fn on_dirty(&self, owner: &CoreHandle, dirt: ComponentDirt) {
        if has_dirt(
            dirt,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            self.notify_shape();
        }
        // pathChanged may enter a renderer and retire this Path; the dirt add
        // below rechecks its liveness.
        if self.deferred.get() {
            self.add_dirt(owner, ComponentDirt::PATH);
        }
    }

    /// Path::markPathDirty's own part: addDirt(Path), then shapePathChanged.
    pub(crate) fn mark_path_dirty(&self, owner: &CoreHandle) {
        self.add_dirt(owner, ComponentDirt::PATH);
        if owner.is_occupied() {
            self.notify_shape();
        }
    }

    /// Path::shapePathChanged.
    fn notify_shape(&self) {
        if let Some(shape) = self.take_shape_notification() {
            shape.cells.path_changed(&shape.handle);
        }
    }

    fn add_dirt(&self, owner: &CoreHandle, value: ComponentDirt) {
        if self
            .dirt
            .with_graph(|graph, index| graph.add_dirt(index, value, false))
            .is_none()
        {
            ComponentOccurrenceHandle::Authored(owner.clone()).add_dirt(value, false);
        }
    }
}
