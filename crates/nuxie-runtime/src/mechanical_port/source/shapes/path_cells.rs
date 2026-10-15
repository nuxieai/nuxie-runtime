//! The state of a Path that the dirt cascade, its update and its shape's
//! PathComposer share: its shape link, the once-per-update shape notification,
//! its deferred rebuild, its flags, its built geometry and its transform. Held
//! outside the Path so Path::onDirty, markPathDirty and the composer's
//! per-path reads run without borrowing it.

use std::{
    cell::{Cell, OnceCell, Ref, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    component::{ComponentOccurrenceHandle, SharedDirt, has_dirt},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    math::{mat2d::Mat2D, raw_path::RawPath},
    shapes::{path_flags::PathFlags, shape_cells::ShapeCells},
    world_transform_component::TransformCells,
};

pub(crate) struct PathCells {
    // The path's own Component dirt.
    dirt: Rc<SharedDirt>,
    // The path's own transform; its world is the path transform unless skinned.
    transform: Rc<TransformCells>,
    // Set once, when the path joins its Shape.
    shape: OnceCell<PathShape>,
    // Whether this update cycle already told the shape its path changed.
    shape_notified: Cell<bool>,
    // The last update skipped the rebuild while the shape was transparent.
    deferred: Cell<bool>,
    // Runtime flags (followPath, clipping), separate from the authored ones.
    flags: Cell<PathFlags>,
    // The authored pathFlags property; bit 0 hides the path.
    authored_flags: Cell<u32>,
    // A skinned PointsPath's vertices are already in world space.
    skinned: Cell<bool>,
    // Bumped by each rebuild of raw_path.
    geometry_version: Cell<u32>,
    raw_path: RefCell<RawPath>,
}

#[derive(Clone)]
pub(crate) struct PathShape {
    pub(crate) handle: CoreHandle,
    pub(crate) cells: Rc<ShapeCells>,
}

impl PathCells {
    pub(crate) fn new(dirt: Rc<SharedDirt>, transform: Rc<TransformCells>) -> Self {
        Self {
            dirt,
            transform,
            shape: OnceCell::new(),
            shape_notified: Cell::new(false),
            deferred: Cell::new(false),
            flags: Cell::new(PathFlags::NONE),
            authored_flags: Cell::new(0),
            skinned: Cell::new(false),
            geometry_version: Cell::new(0),
            raw_path: RefCell::new(RawPath::default()),
        }
    }

    pub(crate) fn authored_flags(&self) -> u32 {
        self.authored_flags.get()
    }

    pub(crate) fn set_authored_flags(&self, flags: u32) {
        self.authored_flags.set(flags);
    }

    /// Path::isHidden(): the authored hidden bit.
    #[inline]
    pub(crate) fn is_hidden(&self) -> bool {
        self.authored_flags.get() & 1 == 1
    }

    /// Component::isCollapsed() for this path.
    #[inline]
    pub(crate) fn is_collapsed(&self) -> bool {
        self.dirt.get().contains(ComponentDirt::COLLAPSED)
    }

    pub(crate) fn set_skinned(&self) {
        self.skinned.set(true);
    }

    /// Path::pathTransform(): identity for a skinned PointsPath, else world.
    #[inline]
    pub(crate) fn path_transform(&self) -> Mat2D {
        if self.skinned.get() {
            Mat2D::identity()
        } else {
            self.transform.world()
        }
    }

    pub(crate) fn geometry_version(&self) -> u32 {
        self.geometry_version.get()
    }

    pub(crate) fn bump_geometry_version(&self) {
        self.geometry_version
            .set(self.geometry_version.get().wrapping_add(1));
    }

    pub(crate) fn raw_path(&self) -> Ref<'_, RawPath> {
        self.raw_path.borrow()
    }

    /// Take the built path out for a rebuild; `put_raw_path` returns it.
    pub(crate) fn take_raw_path(&self) -> RawPath {
        self.raw_path.take()
    }

    pub(crate) fn put_raw_path(&self, path: RawPath) {
        *self.raw_path.borrow_mut() = path;
    }

    pub(crate) fn shape(&self) -> Option<PathShape> {
        self.shape.get().cloned()
    }

    pub(crate) fn shape_handle(&self) -> Option<CoreHandle> {
        self.shape.get().map(|shape| shape.handle.clone())
    }

    /// Path::deformer().is_some(): the joined shape is live and deformed.
    #[inline]
    pub(crate) fn has_deformer(&self) -> bool {
        self.shape
            .get()
            .is_some_and(|shape| shape.handle.is_occupied() && shape.cells.has_deformer())
    }

    /// Path::onAddedClean's shape link; a path joins one Shape.
    pub(crate) fn bind_shape(&self, handle: CoreHandle, cells: Rc<ShapeCells>) {
        if let Err(PathShape { handle, .. }) = self.shape.set(PathShape { handle, cells }) {
            debug_assert!(self.shape_handle().is_some_and(|bound| bound == handle));
        }
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
        self.shape.get().is_some_and(|shape| {
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
        self.on_dirty_with(|| owner, dirt);
    }

    /// on_dirty, reading the path's own handle only when it is needed.
    #[inline]
    pub(crate) fn on_dirty_with<'a>(
        &self,
        owner: impl FnOnce() -> &'a CoreHandle,
        dirt: ComponentDirt,
    ) {
        if has_dirt(
            dirt,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            self.notify_shape();
        }
        // pathChanged may enter a renderer and retire this Path; the dirt add
        // below rechecks its liveness.
        if self.deferred.get() {
            self.add_dirt(owner(), ComponentDirt::PATH);
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
        if self.shape_notified.get() {
            return;
        }
        let Some(shape) = self.shape.get() else {
            return;
        };
        self.shape_notified.set(true);
        shape.cells.path_changed(&shape.handle);
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
