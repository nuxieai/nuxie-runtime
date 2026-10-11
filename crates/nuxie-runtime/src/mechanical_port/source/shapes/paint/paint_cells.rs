//! The state of a Fill or Stroke that its container's path pipeline and the
//! pathChanged cascade read every frame. The paint and its container share it,
//! so those readers never borrow the paint's core object.

use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    component::SharedDirt,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    shapes::{
        paint::{
            effects_container,
            shape_paint_path::ShapePaintPath,
            stroke_effect::{EffectPaths, PathProvider, invalidate_effect_paths},
            shape_paint::{RuntimeRenderPaintHandle, ShapePaintType},
        },
        path_flags::PathFlags,
        shape_paint_container::ContainerCells,
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
    effect_paths: RefCell<Vec<(CoreHandle, EffectPaths)>>,
    has_other_effects: Cell<bool>,
    has_feather: Cell<bool>,
    // Created once, when the paint mutator first initializes the paint.
    render_paint: OnceCell<RuntimeRenderPaintHandle>,
    // The paint's own Component dirt.
    dirt: Rc<SharedDirt>,
    // The container whose flag union includes this paint's flags.
    container: RefCell<Weak<ContainerCells>>,
}

impl PaintCells {
    pub(crate) fn new(paint_type: ShapePaintType, dirt: Rc<SharedDirt>) -> Self {
        Self {
            paint_type,
            path_flags: Cell::new(PathFlags::NONE),
            effect_paths: RefCell::new(Vec::new()),
            has_other_effects: Cell::new(false),
            has_feather: Cell::new(false),
            render_paint: OnceCell::new(),
            dirt,
            container: RefCell::new(Weak::new()),
        }
    }

    #[inline]
    pub(crate) fn path_flags(&self) -> PathFlags {
        self.path_flags.get()
    }

    pub(crate) fn set_path_flags(&self, flags: PathFlags) {
        if self.path_flags.replace(flags) != flags {
            if let Some(container) = self.container.borrow().upgrade() {
                container.refresh_paint_flags();
            }
        }
    }

    pub(crate) fn set_container(&self, container: Weak<ContainerCells>) {
        *self.container.borrow_mut() = container;
    }

    /// Record an added stroke effect, with its paths when invalidating it
    /// only rewinds them.
    pub(crate) fn add_effect(&self, effect: CoreHandle, paths: Option<EffectPaths>) {
        match paths {
            Some(paths) => self.effect_paths.borrow_mut().push((effect, paths)),
            None => self.has_other_effects.set(true),
        }
    }

    /// EffectsContainer::lastEffectPath(provider) when every effect keeps its
    /// paths here; None when another kind of effect must answer for itself.
    pub(crate) fn last_effect_path(
        &self,
        provider: &PathProvider,
    ) -> Option<Option<Rc<RefCell<ShapePaintPath>>>> {
        if self.has_other_effects.get() {
            return None;
        }
        Some(
            self.effect_paths
                .borrow()
                .iter()
                .rev()
                .filter(|(effect, _)| effect.is_occupied())
                .find_map(|(_, paths)| {
                    paths
                        .borrow_mut()
                        .get_mut(&provider.identity())
                        .and_then(|path| path.path())
                }),
        )
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

    #[inline]
    pub(crate) fn render_paint(&self) -> Option<&RuntimeRenderPaintHandle> {
        self.render_paint.get()
    }

    /// Install the render paint; a paint is initialized at most once.
    pub(crate) fn set_render_paint(&self, paint: RuntimeRenderPaintHandle) {
        assert!(
            self.render_paint.set(paint).is_ok(),
            "a ShapePaint initializes its render paint once"
        );
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
        for (_, paths) in self.effect_paths.borrow().iter() {
            invalidate_effect_paths(paths, None);
        }
        // Stroke::invalidateRendering's prefix, through the installed paint
        // (ShapePaintRenderingInvalidation::stroke, without retaining it).
        if self.paint_type == ShapePaintType::Stroke {
            match self.render_paint() {
                Some(paint) => paint.borrow_mut().invalidate_stroke(),
                #[cfg(feature = "editor")]
                None => return,
                #[cfg(not(feature = "editor"))]
                None => panic!("initialized Stroke render paint"),
            }
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
