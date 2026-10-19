//! The state that Shape::pathChanged, the PathComposer's once-per-update gates
//! and the paths' deferral read and write every frame. A Shape, its composer
//! and its paths share it, so the dirt cascade reaches it without borrowing
//! the Shape.

use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    component::{ComponentOccurrenceHandle, SharedDirt},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    shapes::{
        path_cells::PathCells, path_flags::PathFlags, shape::Shape,
        shape_paint_container::ContainerCells,
    },
    world_transform_component::TransformCells,
};

/// One of a shape's paths: its handle (for liveness and the rare borrowing
/// reads) and its cells.
#[derive(Clone)]
pub(crate) struct ShapePath {
    pub(crate) handle: CoreHandle,
    pub(crate) cells: Rc<PathCells>,
}

pub(crate) struct ShapeCells {
    // The installed Shape; pathChanged's rarer steps still borrow it.
    shape: OnceCell<CoreHandle>,
    // The shape's paths, in order. Replaced, never edited, when one is added.
    paths: RefCell<Rc<[ShapePath]>>,
    transform: Rc<TransformCells>,
    paints: Rc<ContainerCells>,
    // The PathComposer's Component dirt and occurrence, and its gates: whether
    // its last update deferred, and whether it told the shape since.
    composer_dirt: Rc<SharedDirt>,
    composer: ComponentOccurrenceHandle,
    composer_deferred: Cell<bool>,
    composer_notified: Cell<bool>,
    // The NSlicedNode ancestor deforming this shape's paths, if any.
    deformer: RefCell<Option<CoreHandle>>,
    world_length: Cell<f32>,
    local_bounds_clean: Cell<bool>,
    has_layout_participant: Cell<bool>,
    // The installed Shape's DrawableBase drawableFlags storage.
    drawable_flags: OnceCell<Rc<Cell<u16>>>,
}

impl ShapeCells {
    pub(crate) fn new(
        transform: Rc<TransformCells>,
        paints: Rc<ContainerCells>,
        composer_dirt: Rc<SharedDirt>,
        composer: ComponentOccurrenceHandle,
    ) -> Self {
        Self {
            shape: OnceCell::new(),
            paths: RefCell::new(Rc::new([])),
            transform,
            paints,
            composer_dirt,
            composer,
            composer_deferred: Cell::new(false),
            composer_notified: Cell::new(false),
            deformer: RefCell::new(None),
            world_length: Cell::new(-1.0),
            local_bounds_clean: Cell::new(false),
            has_layout_participant: Cell::new(false),
            drawable_flags: OnceCell::new(),
        }
    }

    pub(crate) fn shape(&self) -> Option<CoreHandle> {
        self.shape.get().cloned()
    }

    /// The installed Shape; a composer belongs to one Shape.
    pub(crate) fn bind_shape(&self, shape: CoreHandle) {
        if let Err(shape) = self.shape.set(shape) {
            debug_assert!(self.shape.get() == Some(&shape));
        }
    }

    /// The installed Shape's drawableFlags storage.
    pub(crate) fn bind_drawable_flags(&self, flags: Rc<Cell<u16>>) {
        let _ = self.drawable_flags.set(flags);
    }

    /// The bounds marking a PathComposer update owes its Shape
    /// (Shape::markBoundsDirty when the measured geometry changed, else
    /// markWorldBoundsDirty), done on these cells. False, having done
    /// nothing, when the shape's own marking is needed: a layout participant
    /// is told of measured geometry, or the flags are not bound.
    pub(crate) fn mark_bounds_dirty_from_composer(
        &self,
        shape: &CoreHandle,
        measured_geometry_changed: bool,
    ) -> bool {
        let Some(flags) = self.drawable_flags.get() else {
            return false;
        };
        if measured_geometry_changed && self.has_layout_participant.get() {
            return false;
        }
        // Shape::markWorldBoundsDirty: setDrawableFlags, then the world length.
        let value = flags.get()
            & !crate::mechanical_port::source::drawable_flag::DrawableFlag::WORLD_BOUNDS_CLEAN.0;
        if flags.get() != value {
            flags.set(value);
            shape.notify_property_changed(
                crate::mechanical_port::source::generated::drawable_base::DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY,
            );
        }
        self.world_length.set(-1.0);
        if measured_geometry_changed {
            self.local_bounds_clean.set(false);
        }
        true
    }

    pub(crate) fn paths(&self) -> Rc<[ShapePath]> {
        Rc::clone(&self.paths.borrow())
    }

    pub(crate) fn add_path(&self, handle: CoreHandle, cells: Rc<PathCells>) {
        let paths: Rc<[ShapePath]> = self
            .paths
            .borrow()
            .iter()
            .cloned()
            .chain(std::iter::once(ShapePath { handle, cells }))
            .collect();
        *self.paths.borrow_mut() = paths;
    }

    /// Shape::isEmpty(): every path hidden, collapsed or gone.
    pub(crate) fn is_empty(&self) -> bool {
        self.paths.borrow().iter().all(|path| {
            !path.handle.is_occupied() || path.cells.is_hidden() || path.cells.is_collapsed()
        })
    }

    pub(crate) fn paints(
        &self,
    ) -> Rc<[crate::mechanical_port::source::shapes::shape_paint_container::ContainerPaint]> {
        self.paints.paints()
    }

    /// The composer of a drawing shape, which the shape keeps alive.
    fn drawing_composer(
        &self,
    ) -> Rc<std::cell::RefCell<crate::mechanical_port::source::shapes::path_composer::PathComposer>>
    {
        let ComponentOccurrenceHandle::PathComposer(composer) = &self.composer else {
            unreachable!("a shape's composer occurrence is a PathComposer");
        };
        composer
            .upgrade()
            .expect("a drawing shape keeps its composer")
    }

    fn composer_path_mut<R>(
        composer: &std::cell::RefCell<
            crate::mechanical_port::source::shapes::path_composer::PathComposer,
        >,
        kind: crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaintPathKind,
        use_path: impl FnOnce(
            &mut crate::mechanical_port::source::shapes::paint::shape_paint_path::ShapePaintPath,
        ) -> R,
    ) -> R {
        use crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaintPathKind;
        let mut composer = composer.borrow_mut();
        use_path(match kind {
            ShapePaintPathKind::Local => composer.local_path(),
            ShapePaintPathKind::LocalClockwise => composer.local_clockwise_path(),
            ShapePaintPathKind::World => composer.world_path(),
        })
    }

    /// Shape::draw: each visible paint over its path. A plain paint draws
    /// from its cells; any other is borrowed and drawn by its ShapePaint.
    pub(crate) fn draw(
        &self,
        needs_save_operation: bool,
        renderer: &mut crate::mechanical_port::source::renderer::Renderer,
        factory: &dyn Fn() -> crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    ) {
        let paints = self.paints.paints();
        let needs_save = needs_save_operation || paints.len() > 1;
        let transform = self.transform.world();
        // One upgrade of the composer for all the paints.
        let composer = self.drawing_composer();
        for paint in paints.iter() {
            if !paint.cells.is_visible() {
                continue;
            }
            let kind = paint.cells.path_kind();
            let fill_rule = paint.cells.fill_rule();
            if paint.cells.is_plain() && paint.handle.is_occupied_settled() {
                Self::composer_path_mut(&composer, kind, |path| {
                    paint
                        .cells
                        .draw_plain(renderer, path, transform, needs_save, fill_rule, factory);
                });
                continue;
            }
            paint.handle.with_mut(|object| {
                let Some(behavior) = object.as_shape_paint_behavior_mut() else {
                    return;
                };
                Self::composer_path_mut(&composer, kind, |path| {
                    behavior.shape_paint_mut().draw_with_active_container(
                        renderer,
                        path,
                        transform,
                        false,
                        None,
                        needs_save,
                        fill_rule,
                        &|| transform,
                        None,
                    );
                });
            });
        }
    }

    /// Run `use_path` on the composer's world path, if the composer lives.
    pub(crate) fn with_world_path<R>(
        &self,
        use_path: impl FnOnce(
            &mut crate::mechanical_port::source::shapes::paint::shape_paint_path::ShapePaintPath,
        ) -> R,
    ) -> Option<R> {
        let ComponentOccurrenceHandle::PathComposer(composer) = &self.composer else {
            unreachable!("a shape's composer occurrence is a PathComposer");
        };
        let composer = composer.upgrade()?;
        let mut composer = composer.borrow_mut();
        Some(use_path(composer.world_path()))
    }

    /// The shape's world transform.
    pub(crate) fn world(&self) -> crate::mechanical_port::source::math::mat2d::Mat2D {
        self.transform.world()
    }

    pub(crate) fn is_flagged(&self, flags: PathFlags) -> bool {
        self.paints.is_flagged(flags)
    }

    pub(crate) fn path_flags(&self) -> PathFlags {
        self.paints.path_flags()
    }

    pub(crate) fn render_opacity(&self) -> f32 {
        self.transform.render_opacity()
    }

    /// Shape::canDeferPathUpdate().
    pub(crate) fn can_defer_path_update(&self) -> bool {
        self.transform.render_opacity() == 0.0
            && !self.is_flagged(PathFlags::CLIPPING | PathFlags::NEVER_DEFER_UPDATE)
    }

    pub(crate) fn deformer(&self) -> Option<CoreHandle> {
        self.deformer.borrow().clone()
    }

    pub(crate) fn has_deformer(&self) -> bool {
        self.deformer.borrow().is_some()
    }

    pub(crate) fn set_deformer(&self, deformer: Option<CoreHandle>) {
        *self.deformer.borrow_mut() = deformer;
    }

    pub(crate) fn world_length(&self) -> f32 {
        self.world_length.get()
    }

    pub(crate) fn set_world_length(&self, length: f32) {
        self.world_length.set(length);
    }

    pub(crate) fn local_bounds_clean(&self) -> bool {
        self.local_bounds_clean.get()
    }

    pub(crate) fn set_local_bounds_clean(&self, clean: bool) {
        self.local_bounds_clean.set(clean);
    }

    pub(crate) fn has_layout_participant(&self) -> bool {
        self.has_layout_participant.get()
    }

    pub(crate) fn set_has_layout_participant(&self) {
        self.has_layout_participant.set(true);
    }

    pub(crate) fn composer_deferred(&self) -> bool {
        self.composer_deferred.get()
    }

    pub(crate) fn set_composer_deferred(&self, deferred: bool) {
        self.composer_deferred.set(deferred);
    }

    pub(crate) fn composer_notified(&self) -> bool {
        self.composer_notified.get()
    }

    pub(crate) fn set_composer_notified(&self, notified: bool) {
        self.composer_notified.set(notified);
    }

    /// PathComposer::onDirty: a composer whose last update deferred tells its
    /// shape the path changed, once per update.
    pub(crate) fn composer_on_dirty(&self) {
        if !self.composer_deferred.get() || self.composer_notified.get() {
            return;
        }
        let shape = self.shape.get().expect("arena-installed Shape");
        self.composer_notified.set(true);
        self.path_changed(shape);
    }

    /// Shape::pathChanged() for a caller that holds no Shape borrow: a path,
    /// a path's skin or the composer.
    pub(crate) fn path_changed(&self, shape: &CoreHandle) {
        if !shape.is_occupied() {
            return;
        }
        self.add_composer_dirt(ComponentDirt::PATH, true);
        // The composer's dependents may retire the shape.
        if !shape.is_occupied() {
            return;
        }
        self.world_length.set(-1.0);
        // Geometry and path transforms invalidate this even when a transparent
        // shape defers the composer update and therefore mark_bounds_dirty.
        self.local_bounds_clean.set(false);
        if self.has_layout_participant.get() {
            shape.with_downcast_mut::<Shape, _>(Shape::invalidate_intrinsic_bounds);
        }
        if self.transform.has_constraints() {
            let mut index = 0;
            while let Some(constraint) = shape
                .with_downcast::<Shape, _>(|shape| shape.base.constraints().get(index).cloned())
                .flatten()
            {
                ComponentOccurrenceHandle::Authored(constraint)
                    .add_dirt(ComponentDirt::PATH, false);
                index += 1;
            }
        }
        if shape.is_occupied() {
            self.paints.invalidate_stroke_effects();
        }
    }

    fn add_composer_dirt(&self, value: ComponentDirt, recurse: bool) -> bool {
        self.composer_dirt
            .with_graph(|graph, index| graph.add_dirt(index, value, recurse))
            .unwrap_or_else(|| self.composer.add_dirt(value, recurse))
    }
}
