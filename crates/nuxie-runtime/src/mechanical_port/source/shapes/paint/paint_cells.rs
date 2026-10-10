//! The state of a Fill or Stroke that its container's path pipeline and the
//! pathChanged cascade read every frame. The paint and its container share it,
//! so those readers never borrow the paint's core object.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    component::SharedDirt,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    shapes::{
        paint::{
            effects_container,
            shape_paint::{
                RuntimeRenderPaintHandle, ShapePaintRenderingInvalidation, ShapePaintType,
            },
        },
        path_flags::PathFlags,
    },
};

pub(crate) struct PaintCells {
    paint_type: ShapePaintType,
    // Fill: local, or local-clockwise under the clockwise fill rule. Stroke:
    // local when transformAffectsStroke, world otherwise. The owning
    // property's storage writes it.
    path_flags: Cell<PathFlags>,
    // Stroke effects or a feather give invalidateEffects work beyond
    // invalidateRendering.
    has_effects: Cell<bool>,
    has_feather: Cell<bool>,
    render_paint: RefCell<Option<RuntimeRenderPaintHandle>>,
    // The paint's own Component dirt.
    dirt: Rc<SharedDirt>,
}

impl PaintCells {
    pub(crate) fn new(paint_type: ShapePaintType, dirt: Rc<SharedDirt>) -> Self {
        Self {
            paint_type,
            path_flags: Cell::new(PathFlags::NONE),
            has_effects: Cell::new(false),
            has_feather: Cell::new(false),
            render_paint: RefCell::new(None),
            dirt,
        }
    }

    #[inline]
    pub(crate) fn path_flags(&self) -> PathFlags {
        self.path_flags.get()
    }

    pub(crate) fn set_path_flags(&self, flags: PathFlags) {
        self.path_flags.set(flags);
    }

    pub(crate) fn set_has_effects(&self) {
        self.has_effects.set(true);
    }

    pub(crate) fn set_has_feather(&self) {
        self.has_feather.set(true);
    }

    pub(crate) fn render_paint(&self) -> Option<RuntimeRenderPaintHandle> {
        self.render_paint.borrow().clone()
    }

    pub(crate) fn set_render_paint(&self, paint: RuntimeRenderPaintHandle) {
        *self.render_paint.borrow_mut() = Some(paint);
    }

    /// ShapePaint::invalidateEffects() as pathChanged runs it on each of the
    /// container's paints. Without effects or a feather it is only
    /// invalidateRendering, which these cells carry out on their own.
    pub(crate) fn invalidate_effects(&self, paint: &CoreHandle) {
        if self.has_effects.get() || self.has_feather.get() {
            effects_container::invalidate_effects_handle(paint, None);
            return;
        }
        let graph = self.dirt.graph_binding();
        let live = match &graph {
            Some((graph, index)) => graph.is_live(*index),
            None => paint.is_alive(),
        };
        if !live {
            return;
        }
        let invalidation = match self.paint_type {
            ShapePaintType::Fill => ShapePaintRenderingInvalidation::Base,
            ShapePaintType::Stroke => ShapePaintRenderingInvalidation::stroke(self.render_paint()),
        };
        if !invalidation.before_dirt() {
            return;
        }
        // ShapePaint::invalidateRendering's addDirt(Path, true).
        if let Some((graph, index)) = graph {
            graph.add_dirt(index, ComponentDirt::PATH, true);
        } else if !self.dirt.get().contains(ComponentDirt::PATH) {
            effects_container::add_native_path_dirt(paint, self.paint_type);
        }
    }
}
