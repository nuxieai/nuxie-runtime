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
            stroke_effect::{EffectPaths, invalidate_effect_paths},
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
    // The paths of effects whose invalidateEffect only rewinds them, in
    // effect order. Any other effect, or a feather, sends invalidateEffects
    // through the general path.
    effect_paths: RefCell<Vec<EffectPaths>>,
    has_other_effects: Cell<bool>,
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
            effect_paths: RefCell::new(Vec::new()),
            has_other_effects: Cell::new(false),
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

    /// Record an added stroke effect, with its paths when invalidating it
    /// only rewinds them.
    pub(crate) fn add_effect(&self, paths: Option<EffectPaths>) {
        match paths {
            Some(paths) => self.effect_paths.borrow_mut().push(paths),
            None => self.has_other_effects.set(true),
        }
    }

    pub(crate) fn paint_type(&self) -> ShapePaintType {
        self.paint_type
    }

    /// EffectsContainer::hasEffects().
    pub(crate) fn has_effects(&self) -> bool {
        self.has_other_effects.get() || !self.effect_paths.borrow().is_empty()
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
    /// container's paints: rewind the effects' paths, then invalidateRendering.
    /// A feather or an effect with its own invalidation takes the general path.
    pub(crate) fn invalidate_effects(&self, paint: &CoreHandle) {
        if self.has_other_effects.get() || self.has_feather.get() {
            effects_container::invalidate_effects_handle(paint, None);
            return;
        }
        let live = self
            .dirt
            .with_graph(|graph, index| graph.is_live(index))
            .unwrap_or_else(|| paint.is_occupied());
        if !live {
            return;
        }
        for paths in self.effect_paths.borrow().iter() {
            invalidate_effect_paths(paths, None);
        }
        let invalidation = match self.paint_type {
            ShapePaintType::Fill => ShapePaintRenderingInvalidation::Base,
            ShapePaintType::Stroke => ShapePaintRenderingInvalidation::stroke(self.render_paint()),
        };
        if !invalidation.before_dirt() {
            return;
        }
        // ShapePaint::invalidateRendering's addDirt(Path, true).
        let added = self
            .dirt
            .with_graph(|graph, index| graph.add_dirt(index, ComponentDirt::PATH, true));
        if added.is_none() && !self.dirt.get().contains(ComponentDirt::PATH) {
            effects_container::add_native_path_dirt(paint, self.paint_type);
        }
    }
}
