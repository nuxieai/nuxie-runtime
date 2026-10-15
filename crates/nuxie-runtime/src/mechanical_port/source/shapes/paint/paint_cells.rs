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
    factory::RuntimeFactoryHandle,
    math::mat2d::Mat2D,
    shapes::{
        paint::{
            effects_container,
            feather::FeatherCells,
            shape_paint_path::ShapePaintPath,
            stroke_effect::{EffectPaths, PathProvider, invalidate_effect_paths},
            shape_paint::{RuntimeRenderPaintHandle, ShapePaintPathKind, ShapePaintType},
            shape_paint_mutator::MutatorFlags,
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
    // The installed feather and its cells.
    feather: RefCell<Option<(CoreHandle, Rc<FeatherCells>)>>,
    // Created once, when the paint mutator first initializes the paint.
    render_paint: OnceCell<RuntimeRenderPaintHandle>,
    // The flags of the mutator that initialized the render paint, shared by
    // that mutator.
    mutator_flags: OnceCell<Rc<Cell<MutatorFlags>>>,
    // The paint's own Component dirt.
    dirt: Rc<SharedDirt>,
    // The container whose flag union includes this paint's flags.
    container: RefCell<Weak<ContainerCells>>,
    // ShapePaintBase's isVisible storage.
    is_visible: Rc<Cell<bool>>,
    // Stroke's thickness and transformAffectsStroke, Fill's fillRule; their
    // owning properties' storage writes them.
    thickness: Cell<f32>,
    transform_affects_stroke: Cell<bool>,
    fill_rule: Cell<u8>,
    // The paint's PathProvider identity, fixed at construction.
    provider: PathProvider,
    // ShapePaint's isFill, hasPaintImage and hasModulatedImage, written where
    // the paint writes them.
    is_fill: Cell<bool>,
    has_paint_image: Cell<bool>,
    has_modulated_image: Cell<bool>,
}

impl PaintCells {
    pub(crate) fn new(
        paint_type: ShapePaintType,
        dirt: Rc<SharedDirt>,
        is_visible: Rc<Cell<bool>>,
        provider: PathProvider,
    ) -> Self {
        Self {
            paint_type,
            path_flags: Cell::new(PathFlags::NONE),
            effect_paths: RefCell::new(Vec::new()),
            has_other_effects: Cell::new(false),
            has_feather: Cell::new(false),
            feather: RefCell::new(None),
            render_paint: OnceCell::new(),
            mutator_flags: OnceCell::new(),
            dirt,
            container: RefCell::new(Weak::new()),
            is_visible,
            thickness: Cell::new(1.0),
            transform_affects_stroke: Cell::new(true),
            fill_rule: Cell::new(0),
            provider,
            is_fill: Cell::new(false),
            has_paint_image: Cell::new(false),
            has_modulated_image: Cell::new(false),
        }
    }

    #[inline]
    pub(crate) fn set_is_fill(&self, value: bool) {
        self.is_fill.set(value);
    }

    #[inline]
    pub(crate) fn set_has_paint_image(&self, value: bool) {
        self.has_paint_image.set(value);
    }

    #[inline]
    pub(crate) fn set_has_modulated_image(&self, value: bool) {
        self.has_modulated_image.set(value);
    }

    /// A paint with no inner feather, no effect beyond those that only
    /// rewind their path, and no image: ShapePaint::draw needs nothing but
    /// these cells (and an outer feather's).
    #[inline]
    pub(crate) fn is_plain(&self) -> bool {
        !self.has_other_effects.get()
            && !self.has_paint_image.get()
            && !self.has_modulated_image.get()
            && (!self.has_feather.get()
                || self
                    .feather
                    .borrow()
                    .as_ref()
                    .is_some_and(|(_, feather)| !feather.is_inner()))
    }

    /// The feather's cells while its occurrence lives; a retired feather
    /// draws nothing, as its loan would fail.
    #[inline]
    pub(crate) fn live_feather(&self) -> Option<Rc<FeatherCells>> {
        self.feather
            .borrow()
            .as_ref()
            .filter(|(handle, _)| handle.is_occupied())
            .map(|(_, cells)| Rc::clone(cells))
    }

    /// ShapePaint::draw for a plain paint as a Shape draws it (its own fill
    /// rule, no override paint): the same renderer calls, from these cells.
    pub(crate) fn draw_plain(
        &self,
        renderer: &mut dyn nuxie_render_api::Renderer,
        shape_paint_path: &mut ShapePaintPath,
        transform: Mat2D,
        needs_save_operation: bool,
        fill_rule: Option<u32>,
        factory: &dyn Fn() -> RuntimeFactoryHandle,
    ) {
        let mut saved = !needs_save_operation;
        // An outer feather offsets the drawing: in artboard space before the
        // path's local transform, in local space after it.
        let feather = if self.has_feather.get() { self.live_feather() } else { None };
        let offset = |world: bool| {
            feather
                .as_ref()
                .filter(|feather| feather.space_is_world() == world && feather.has_offset())
        };
        if offset(true).is_some() {
            if !saved {
                saved = true;
                renderer.save();
            }
            // save() may change the feather: translate by its current offset.
            let feather = feather.as_ref().expect("an offset feather");
            renderer.translate(feather.offset_x(), feather.offset_y());
        }
        if shape_paint_path.is_local() {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
        }
        let path_effect = self
            .last_effect_path(&self.provider)
            .expect("a plain paint has only rewinding effects");
        if offset(false).is_some() {
            if !saved {
                saved = true;
                renderer.save();
            }
            let feather = feather.as_ref().expect("an offset feather");
            renderer.translate(feather.offset_x(), feather.offset_y());
        }
        let mut draw_path = |path: &mut ShapePaintPath| {
            let render_path = path.render_path_with_deferred_factory(factory);
            if self.is_fill.get() {
                if let Some(fill_rule) = fill_rule {
                    match fill_rule {
                        0 => render_path.fill_rule(nuxie_render_api::FillRule::NonZero),
                        1 => render_path.fill_rule(nuxie_render_api::FillRule::EvenOdd),
                        2 => render_path.fill_rule(nuxie_render_api::FillRule::Clockwise),
                        _ => {}
                    }
                }
            }
            let render_path = path.existing_render_path();
            if let Some(paint) = self.render_paint() {
                renderer.draw_path(render_path, paint.borrow().as_ref());
            }
        };
        if let Some(effect) = path_effect {
            draw_path(&mut effect.borrow_mut());
        } else {
            draw_path(shape_paint_path);
        }
        if saved && needs_save_operation {
            renderer.restore();
        }
    }

    #[inline]
    pub(crate) fn set_thickness(&self, value: f32) {
        self.thickness.set(value);
    }

    #[inline]
    pub(crate) fn set_transform_affects_stroke(&self, value: bool) {
        self.transform_affects_stroke.set(value);
    }

    #[inline]
    pub(crate) fn set_fill_rule(&self, value: u8) {
        self.fill_rule.set(value);
    }

    /// ShapePaintBehavior::isVisible: Fill's, or Stroke's (visible with a
    /// positive thickness).
    #[inline]
    pub(crate) fn is_visible(&self) -> bool {
        match self.paint_type {
            ShapePaintType::Fill => self.is_visible.get(),
            ShapePaintType::Stroke => self.is_visible.get() && self.thickness.get() > 0.0,
        }
    }

    /// ShapePaintBehavior::pickPathKind.
    #[inline]
    pub(crate) fn path_kind(&self) -> ShapePaintPathKind {
        match self.paint_type {
            ShapePaintType::Fill => {
                if self.fill_rule.get() == nuxie_render_api::FillRule::Clockwise as u8 {
                    ShapePaintPathKind::LocalClockwise
                } else {
                    ShapePaintPathKind::Local
                }
            }
            ShapePaintType::Stroke => {
                if self.transform_affects_stroke.get() {
                    ShapePaintPathKind::Local
                } else {
                    ShapePaintPathKind::World
                }
            }
        }
    }

    /// ShapePaintBehavior::fillRule: a Fill's rule; none for a Stroke.
    #[inline]
    pub(crate) fn fill_rule(&self) -> Option<u32> {
        match self.paint_type {
            ShapePaintType::Fill => Some(u32::from(self.fill_rule.get())),
            ShapePaintType::Stroke => None,
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
    #[inline]
    pub(crate) fn last_effect_path(
        &self,
        provider: &PathProvider,
    ) -> Option<Option<Rc<RefCell<ShapePaintPath>>>> {
        if self.has_other_effects.get() {
            return None;
        }
        // Most paints have no effects at all.
        if self.effect_paths.borrow().is_empty() {
            return Some(None);
        }
        Some(self.search_effect_paths(provider))
    }

    #[inline(never)]
    fn search_effect_paths(&self, provider: &PathProvider) -> Option<Rc<RefCell<ShapePaintPath>>> {
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
            })
    }

    pub(crate) fn paint_type(&self) -> ShapePaintType {
        self.paint_type
    }

    /// EffectsContainer::hasEffects().
    pub(crate) fn has_effects(&self) -> bool {
        self.has_other_effects.get() || !self.effect_paths.borrow().is_empty()
    }

    /// Install the feather and its cells (ShapePaint::feather).
    pub(crate) fn set_feather(&self, feather: CoreHandle, cells: Rc<FeatherCells>) {
        cells.set_paint_is_fill(self.paint_type == ShapePaintType::Fill);
        *self.feather.borrow_mut() = Some((feather, cells));
        self.has_feather.set(true);
    }

    #[inline]
    /// Share the flags of the mutator that just initialized the render paint.
    pub(crate) fn bind_mutator_flags(&self, flags: Rc<Cell<MutatorFlags>>) {
        assert!(
            self.mutator_flags.set(flags).is_ok(),
            "one mutator initializes a ShapePaint's render paint"
        );
    }

    pub(crate) fn mutator_flags(&self) -> Option<MutatorFlags> {
        self.mutator_flags.get().map(|flags| flags.get())
    }

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
