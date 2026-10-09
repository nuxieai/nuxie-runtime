use crate::mechanical_port::source::{
    component_dirt::{ComponentDirt, has_dirt},
    core::CoreHandle,
    core_context::{CoreContext, StatusCode},
    factory::RuntimeFactoryHandle,
    generated::shapes::paint::shape_paint_base::ShapePaintBase,
    math::mat2d::Mat2D,
    shapes::{
        paint::shape_paint_path::ShapePaintPath,
        paint::{
            effects_container::{EffectsContainer, EffectsContainerState},
            feather::Feather,
            shape_paint_mutator::ShapePaintMutator,
            stroke_effect::{PathProvider, StrokeEffect},
        },
        path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer,
    },
    transform_space::TransformSpace,
};
use nuxie_render_api::{BlendMode, RenderPaint, Renderer};
use std::{cell::RefCell, rc::Rc};

pub type RuntimeRenderPaintHandle = Rc<RefCell<Box<dyn RenderPaint>>>;

/// The callback phase of the live invalidateRendering override. No dirt or
/// dependent state is captured before the renderer has finished this call.
pub enum ShapePaintRenderingInvalidation {
    Base,
    #[cfg(feature = "editor")]
    Skip,
    Stroke(RuntimeRenderPaintHandle),
}

impl ShapePaintRenderingInvalidation {
    pub(crate) fn before_dirt(self) -> bool {
        #[cfg(feature = "editor")]
        if matches!(self, Self::Skip) {
            return false;
        }
        if let Self::Stroke(paint) = self {
            paint.borrow_mut().invalidate_stroke();
        }
        true
    }

    pub(crate) fn stroke(paint: Option<RuntimeRenderPaintHandle>) -> Self {
        #[cfg(feature = "editor")]
        if paint.is_none() {
            return Self::Skip;
        }
        Self::Stroke(paint.expect("initialized Stroke render paint"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapePaintType {
    Fill,
    Stroke,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapePaintPathKind {
    Local,
    LocalClockwise,
    World,
}

pub trait ShapePaintBehavior {
    fn shape_paint(&self) -> &ShapePaint;
    fn shape_paint_mut(&mut self) -> &mut ShapePaint;
    fn path_flags(&self) -> PathFlags;
    fn paint_type(&self) -> ShapePaintType;
    fn pick_path_kind(&self) -> ShapePaintPathKind;
    fn is_visible(&self) -> bool;
    fn prepare_rendering_invalidation(&self) -> ShapePaintRenderingInvalidation {
        ShapePaintRenderingInvalidation::Base
    }
    fn invalidate_rendering(&mut self) {
        self.shape_paint_mut().invalidate_rendering_base();
    }
    fn invalidate_effects_from(&mut self, effect: Option<&CoreHandle>) {
        self.shape_paint_mut()
            .effects_container
            .invalidate_effects(effect);
        self.shape_paint_mut().invalidate_effect_feather();
        self.invalidate_rendering();
    }
    fn should_draw(&self) -> bool {
        self.is_visible() && self.shape_paint().mutator_is_visible()
    }
    fn is_translucent(&self) -> bool {
        !self.is_visible() || self.shape_paint().mutator_is_translucent()
    }
    fn fill_rule(&self) -> Option<u32> {
        None
    }
    fn initialize_render_paint(
        &mut self,
        mutator: CoreHandle,
        factory: &RuntimeFactoryHandle,
    ) -> bool;
    fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32);
}

pub struct ShapePaint {
    pub base: ShapePaintBase,
    pub effects_container: EffectsContainerState,
    // The concrete C++ virtual identity exists before Core registration and is
    // not copied or deserialized with the authored ShapePaintBase properties.
    paint_type: ShapePaintType,
    path_provider: PathProvider,
    render_paint: Option<RuntimeRenderPaintHandle>,
    paint_mutator: Option<CoreHandle>,
    // update skipped measuring the effects while fully transparent. Showing
    // this paint again owes them a run.
    effects_deferred: bool,
    feather: Option<CoreHandle>,
    has_modulated_image: bool,
    has_paint_image: bool,
    is_fill: bool,
    script_paint_scope: Option<Rc<crate::scripting::ScriptPaint>>,
}

impl ShapePaint {
    pub(crate) fn new(paint_type: ShapePaintType) -> Self {
        Self {
            base: ShapePaintBase::default(),
            effects_container: EffectsContainerState::default(),
            paint_type,
            path_provider: PathProvider::default(),
            render_paint: None,
            paint_mutator: None,
            effects_deferred: false,
            feather: None,
            has_modulated_image: false,
            has_paint_image: false,
            is_fill: false,
            script_paint_scope: None,
        }
    }
}

impl ShapePaint {
    /// The actual inherited PathProvider, shared with this paint's effects.
    pub fn path_provider(&self) -> &PathProvider {
        &self.path_provider
    }

    pub fn paint_type(&self) -> ShapePaintType {
        self.paint_type
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        self.has_paint_image = self.base.children().iter().any(|child| {
            child.is_type_of(
                crate::source::generated::shapes::paint::paint_image_base::PaintImageBase::TYPE_KEY,
            )
        });
        self.is_fill = self.base.handle().is_some_and(|handle| {
            handle
                .is_type_of(crate::source::generated::shapes::paint::fill_base::FillBase::TYPE_KEY)
        });
        let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        parent
            .with_mut(|parent| {
                let Some(container) = parent.as_shape_paint_container_mut() else {
                    return StatusCode::MissingObject;
                };
                if self.paint_mutator.is_some() {
                    container.add_paint(this);
                }
                StatusCode::Ok
            })
            .unwrap_or(StatusCode::MissingObject)
    }

    pub fn script_paint_scope(&self) -> std::rc::Weak<crate::scripting::ScriptPaint> {
        self.script_paint_scope
            .as_ref()
            .map(Rc::downgrade)
            .unwrap_or_default()
    }

    pub fn update_with_path_kind(
        &mut self,
        value: ComponentDirt,
        kind: ShapePaintPathKind,
        stroke: Option<(f32, u32, u32)>,
    ) {
        if has_dirt(value, ComponentDirt::PATH) && !self.effects_container.effects.is_empty() {
            let parent = self.base.parent_handle().expect("ShapePaint container");
            // Hidden paints are invalidated again when shown (see
            // set_render_opacity). Clip sources still consume their effect
            // geometry while hidden.
            if self.render_opacity() == 0.0
                && parent
                    .with(|container| {
                        let flags = container
                            .as_shape_paint_container()
                            .expect("ShapePaint container")
                            .path_flags_with_active_paint(
                                &self.base.handle().expect("installed ShapePaint"),
                                match kind {
                                    ShapePaintPathKind::Local => PathFlags::LOCAL,
                                    ShapePaintPathKind::World => PathFlags::WORLD,
                                    ShapePaintPathKind::LocalClockwise => {
                                        PathFlags::LOCAL_CLOCKWISE
                                    }
                                },
                            );
                        (flags & PathFlags::CLIPPING).is_empty()
                    })
                    .expect("live ShapePaint container")
            {
                self.effects_deferred = true;
                return;
            }
            self.effects_deferred = false;
            let mut source = None;
            parent.with_mut(|container| {
                container.with_shape_paint_path_mut(kind, &mut |path| {
                    let mut snapshot =
                        ShapePaintPath::with_fill_rule(path.is_local(), path.fill_rule());
                    *snapshot.mutable_raw_path() = path.raw_path().clone();
                    source = Some(snapshot);
                });
            });
            // Effects consume immutable raw geometry. Release the container before
            // a scripted effect reads or edits that same parent TransformComponent.
            let Some(source) = source else {
                return;
            };
            // A paint snapshot is only needed inside an effect invocation.
            // Ordinary Fill/Stroke updates never construct Lua PaintData upstream.
            self.script_paint_scope = Some(Rc::new(crate::scripting::ScriptPaint::from_fresh(
                self, stroke,
            )));
            let mut current: Option<Rc<RefCell<ShapePaintPath>>> = None;
            for handle in self.effects_container.effects.iter().cloned() {
                handle.with_mut(|effect| {
                    let Some(effect) = effect.as_stroke_effect_mut() else {
                        return;
                    };
                    if let Some(current) = current.as_ref() {
                        effect.update_effect(&self.path_provider, &current.borrow(), self);
                    } else {
                        effect.update_effect(&self.path_provider, &source, self);
                    }
                    if let Some(new_path) = effect.effect_path(&self.path_provider) {
                        current = Some(new_path);
                    }
                });
            }
            self.script_paint_scope = None;
        }
    }

    pub fn init_render_paint(
        &mut self,
        mutator: CoreHandle,
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        if self.render_paint.is_some() {
            return false;
        }
        // Upstream uses mutator->component()->artboard()->factory(), not this
        // paint's artboard: the mutator may initialize before its parent paint.
        // The caller snapshots that same factory without reborrowing the active
        // mutator occurrence while the parent Fill/Stroke is borrowed.
        self.paint_mutator = Some(mutator);
        self.render_paint = Some(Rc::new(RefCell::new(
            factory.with_factory_mut(|factory| factory.make_render_paint()),
        )));
        true
    }

    pub fn with_render_paint_mut<R>(
        &mut self,
        use_paint: impl FnOnce(&mut dyn RenderPaint) -> R,
    ) -> Option<R> {
        self.render_paint
            .as_ref()
            .map(|paint| use_paint(paint.borrow_mut().as_mut()))
    }

    pub fn render_paint_handle(&self) -> Option<RuntimeRenderPaintHandle> {
        self.render_paint.clone()
    }

    pub fn inherits_blend_mode(&self) -> bool {
        self.base.blend_mode_value() == 127
    }
    pub fn authored_blend_mode(&self) -> BlendMode {
        match self.base.blend_mode_value() {
            3 => BlendMode::SrcOver,
            12 => BlendMode::Additive,
            14 => BlendMode::Screen,
            15 => BlendMode::Overlay,
            16 => BlendMode::Darken,
            17 => BlendMode::Lighten,
            18 => BlendMode::ColorDodge,
            19 => BlendMode::ColorBurn,
            20 => BlendMode::HardLight,
            21 => BlendMode::SoftLight,
            22 => BlendMode::Difference,
            23 => BlendMode::Exclusion,
            24 => BlendMode::Multiply,
            25 => BlendMode::Hue,
            26 => BlendMode::Saturation,
            27 => BlendMode::Color,
            28 => BlendMode::Luminosity,
            value => panic!("invalid blend mode {value}"),
        }
    }
    pub fn blend_mode(&mut self, parent_value: BlendMode, parent_additive_amount: u8) {
        let inherits = self.inherits_blend_mode();
        let mode = if inherits {
            parent_value
        } else {
            self.authored_blend_mode()
        };
        let amount = if inherits {
            parent_additive_amount
        } else {
            self.base.additive_amount()
        };
        let mut render_paint = self
            .render_paint
            .as_ref()
            .expect("initialized paint")
            .borrow_mut();
        render_paint.blend_mode(mode);
        render_paint.additiveness(super::blend_mode::additiveness_for(mode, amount));
    }
    pub fn additive_amount_changed(&mut self) {
        if self.render_paint.is_none() || self.inherits_blend_mode() {
            return;
        }
        let mode = self.authored_blend_mode();
        let mut render_paint = self.render_paint.as_ref().unwrap().borrow_mut();
        render_paint.blend_mode(mode);
        render_paint.additiveness(super::blend_mode::additiveness_for(
            mode,
            self.base.additive_amount(),
        ));
    }

    pub fn set_feather(&mut self, feather: CoreHandle) {
        self.feather = Some(feather);
    }

    pub fn feather(&self) -> Option<CoreHandle> {
        self.feather.clone()
    }

    pub fn draw_with_fill_rule(
        &mut self,
        renderer: &mut dyn Renderer,
        shape_paint_path: &mut ShapePaintPath,
        transform: Mat2D,
        use_path_fill_rule: bool,
        override_paint: Option<&mut dyn RenderPaint>,
        needs_save_operation: bool,
        fill_rule: Option<u32>,
    ) {
        self.draw_with_context(
            renderer,
            shape_paint_path,
            transform,
            use_path_fill_rule,
            override_paint,
            needs_save_operation,
            fill_rule,
            None,
            None,
        );
    }

    pub fn draw_with_factory(
        &mut self,
        renderer: &mut dyn Renderer,
        shape_paint_path: &mut ShapePaintPath,
        transform: Mat2D,
        use_path_fill_rule: bool,
        override_paint: Option<&mut dyn RenderPaint>,
        needs_save_operation: bool,
        fill_rule: Option<u32>,
        factory: &RuntimeFactoryHandle,
    ) {
        self.draw_with_context(
            renderer,
            shape_paint_path,
            transform,
            use_path_fill_rule,
            override_paint,
            needs_save_operation,
            fill_rule,
            None,
            Some(factory),
        );
    }

    /// Mediate a container that is already borrowed by its draw method. Its
    /// world transform is independent of the matrix used to draw this path.
    /// Invoke its getter only at the inner-feather rebuild. A caller-held
    /// transform owner supplies its stable value; a TextStyle whose Text is
    /// not borrowed instead reads that Text live. The optional factory belongs
    /// to an already-borrowed Artboard.
    pub(crate) fn draw_with_active_container(
        &mut self,
        renderer: &mut dyn Renderer,
        shape_paint_path: &mut ShapePaintPath,
        transform: Mat2D,
        use_path_fill_rule: bool,
        override_paint: Option<&mut dyn RenderPaint>,
        needs_save_operation: bool,
        fill_rule: Option<u32>,
        container_transform: &dyn Fn() -> Mat2D,
        factory: Option<&RuntimeFactoryHandle>,
    ) {
        self.draw_with_context(
            renderer,
            shape_paint_path,
            transform,
            use_path_fill_rule,
            override_paint,
            needs_save_operation,
            fill_rule,
            Some(container_transform),
            factory,
        );
    }

    fn render_path_for_draw<'a>(
        &self,
        path: &'a mut ShapePaintPath,
        active_factory: Option<&RuntimeFactoryHandle>,
    ) -> &'a mut dyn nuxie_render_api::RenderPath {
        if let Some(factory) = active_factory {
            return path.render_path(factory);
        }
        path.render_path_with_deferred_factory(|| {
            self.base
                .with_artboard(|artboard| artboard.factory())
                .flatten()
                .expect("a drawing ShapePaint has its imported Artboard factory")
        })
    }

    fn draw_with_context(
        &mut self,
        renderer: &mut dyn Renderer,
        shape_paint_path: &mut ShapePaintPath,
        transform: Mat2D,
        use_path_fill_rule: bool,
        override_paint: Option<&mut dyn RenderPaint>,
        needs_save_operation: bool,
        fill_rule: Option<u32>,
        active_container_transform: Option<&dyn Fn() -> Mat2D>,
        active_factory: Option<&RuntimeFactoryHandle>,
    ) {
        let mut saved = !needs_save_operation;
        if let Some(feather) = self.feather.as_ref() {
            let offset_in_artboard = feather
                .with_downcast::<Feather, _>(|feather| {
                    feather.space() == TransformSpace::World
                        && !feather.is_inner()
                        && (feather.base.offset_x() != 0.0 || feather.base.offset_y() != 0.0)
                })
                .unwrap_or(false);
            if offset_in_artboard {
                if !saved {
                    saved = true;
                    renderer.save();
                }
                // save() may change the feather. Only the branch decision is
                // retained upstream; the translation uses the current offset.
                if let Some((x, y)) = feather.with_downcast::<Feather, _>(|feather| {
                    (feather.base.offset_x(), feather.base.offset_y())
                }) {
                    renderer.translate(x, y);
                }
            }
        }
        if shape_paint_path.is_local() {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
        }

        let provider = self.path_provider;
        let path_effect = EffectsContainer::last_effect_path(self, &provider);
        let mut inner_path = None;
        if let Some(feather) = self.feather.as_ref() {
            if feather
                .with_downcast::<Feather, _>(Feather::is_inner)
                .unwrap_or(false)
            {
                // Upstream's nonvirtual innerPath() returns its value member's
                // address, represented by this non-null, stable Rc. Retiring
                // the Rust occurrence must still leave our save balanced.
                let Some(path) = feather.with_downcast::<Feather, _>(Feather::inner_path) else {
                    if saved && needs_save_operation {
                        renderer.restore();
                    }
                    return;
                };
                if let Some(effect) = path_effect.as_ref() {
                    if feather
                        .with_downcast::<Feather, _>(Feather::effect_path_dirty)
                        .unwrap_or(false)
                    {
                        let container_transform = active_container_transform
                            .map(|read_transform| read_transform())
                            .or_else(|| {
                                self.base.parent_handle().and_then(|parent| {
                                    parent
                                        .with(|parent| parent.shape_paint_world_transform())
                                        .flatten()
                                })
                            });
                        if let Some(container_transform) = container_transform {
                            feather.with_downcast_mut::<Feather, _>(|feather| {
                                let offset_in_artboard = feather.space() == TransformSpace::World;
                                feather.rebuild_inner_path(
                                    &effect.borrow(),
                                    &container_transform,
                                    offset_in_artboard,
                                );
                            });
                        }
                    }
                }
                inner_path = Some(path);
                if !saved {
                    saved = true;
                    renderer.save();
                }
                // A missing RenderPath resolves the factory after preceding
                // renderer callbacks, at the source's resource creation point.
                if let Some(effect) = path_effect.as_ref() {
                    renderer.clip_path(
                        self.render_path_for_draw(&mut effect.borrow_mut(), active_factory),
                    );
                } else {
                    renderer.clip_path(self.render_path_for_draw(shape_paint_path, active_factory));
                }
            }
            let offset_in_local = feather
                .with_downcast::<Feather, _>(|feather| {
                    feather.space() != TransformSpace::World
                        && !feather.is_inner()
                        && (feather.base.offset_x() != 0.0 || feather.base.offset_y() != 0.0)
                })
                .unwrap_or(false);
            if offset_in_local {
                if !saved {
                    saved = true;
                    renderer.save();
                }
                if let Some((x, y)) = feather.with_downcast::<Feather, _>(|feather| {
                    (feather.base.offset_x(), feather.base.offset_y())
                }) {
                    renderer.translate(x, y);
                }
            }
        }

        let mut draw_path = |path: &mut ShapePaintPath, original: Option<&ShapePaintPath>| {
            let render_path = self.render_path_for_draw(path, active_factory);
            if !use_path_fill_rule && self.is_fill {
                if let Some(fill_rule) = fill_rule {
                    match fill_rule {
                        0 => render_path.fill_rule(nuxie_render_api::FillRule::NonZero),
                        1 => render_path.fill_rule(nuxie_render_api::FillRule::EvenOdd),
                        2 => render_path.fill_rule(nuxie_render_api::FillRule::Clockwise),
                        _ => {}
                    }
                }
            }
            if override_paint.is_none() {
                self.apply_modulated_image(original.unwrap_or(path));
            }
            // C++ retains the selected RenderPath pointer through modulation.
            // Reborrow that resource, without a second factory/path request.
            let render_path = path.existing_render_path();
            if let Some(paint) = override_paint.as_deref() {
                renderer.draw_path(render_path, paint);
            } else if let Some(paint) = self.render_paint.as_ref() {
                renderer.draw_path(render_path, paint.borrow().as_ref());
            }
        };
        if let Some(inner) = inner_path {
            draw_path(&mut inner.borrow_mut(), Some(shape_paint_path));
        } else if let Some(effect) = path_effect {
            draw_path(&mut effect.borrow_mut(), Some(shape_paint_path));
        } else {
            draw_path(shape_paint_path, None);
        }
        if saved && needs_save_operation {
            renderer.restore();
        }
    }

    pub fn invalidate_effects_from(&mut self, effect: Option<&CoreHandle>) {
        self.effects_container.invalidate_effects(effect);
        self.finish_invalidate_effects();
    }

    pub(crate) fn finish_invalidate_effects(&mut self) {
        self.invalidate_effect_feather();
        self.invalidate_rendering();
    }

    pub(crate) fn invalidate_effect_feather(&mut self) {
        if let Some(feather) = self.feather.as_ref() {
            feather.with_downcast_mut::<Feather, _>(|feather| {
                feather.mark_effect_path_dirty();
                // The path we paint changed; an inner feather derives its
                // geometry from that path so it has to rebuild.
                if feather.is_inner() {
                    feather.base.add_dirt(ComponentDirt::PATH, false);
                }
            });
        }
    }

    pub fn invalidate_effects(&mut self) {
        self.invalidate_effects_from(None);
    }

    pub fn invalidate_rendering(&mut self) {
        // An explicit base projection still denotes the live Fill/Stroke.
        // The immutable class selects its override without reborrowing that
        // occurrence; Stroke's qualified superclass call uses the method below.
        if self.prepare_rendering_invalidation().before_dirt() {
            self.invalidate_rendering_base();
        }
    }

    pub(crate) fn prepare_rendering_invalidation(&self) -> ShapePaintRenderingInvalidation {
        match self.paint_type() {
            ShapePaintType::Fill => ShapePaintRenderingInvalidation::Base,
            ShapePaintType::Stroke => {
                ShapePaintRenderingInvalidation::stroke(self.render_paint_handle())
            }
        }
    }

    pub(crate) fn invalidate_rendering_base(&mut self) {
        self.base.add_dirt(ComponentDirt::PATH, true);
    }

    pub(crate) fn feather_handle(&self) -> Option<CoreHandle> {
        self.feather.clone()
    }

    fn apply_modulated_image(&mut self, path: &ShapePaintPath) {
        use crate::source::{
            generated::shapes::paint::paint_image_base::PaintImageBase,
            shapes::paint::paint_image::PaintImage,
        };
        let child = self
            .has_paint_image
            .then(|| {
                self.base
                    .children()
                    .iter()
                    .find(|child| child.is_type_of(PaintImageBase::TYPE_KEY))
                    .cloned()
            })
            .flatten();
        if let Some(child) = child {
            let applied = self
                .with_render_paint_mut(|paint| {
                    child
                        .with_downcast::<PaintImage, _>(|image| {
                            image.apply_to(paint, &path.raw_path().bounds())
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            if applied {
                self.has_modulated_image = true;
                return;
            }
        }
        if self.has_modulated_image {
            self.with_render_paint_mut(|paint| {
                paint.modulated_image(
                    None,
                    nuxie_render_api::ImageSampler::LINEAR_CLAMP,
                    nuxie_render_api::Mat2D::IDENTITY,
                )
            });
            self.has_modulated_image = false;
        }
    }

    pub fn add_stroke_effect(
        &mut self,
        identity: CoreHandle,
        effect: &mut dyn crate::mechanical_port::source::shapes::paint::stroke_effect::StrokeEffect,
    ) {
        effect.add_path_provider(&self.path_provider);
        self.effects_container.add_stroke_effect(identity);
    }

    pub fn render_opacity(&self) -> f32 {
        self.paint_mutator
            .as_ref()
            .and_then(|mutator| {
                mutator.with(|mutator| {
                    mutator
                        .as_shape_paint_mutator()
                        .map(ShapePaintMutator::render_opacity)
                })
            })
            .flatten()
            .unwrap_or(1.0)
    }

    pub fn set_render_opacity(&mut self, value: f32) {
        if let Some(mutator) = self.paint_mutator.as_ref() {
            mutator.with_mut(|mutator| {
                if let Some(mutator) = mutator.as_shape_paint_mutator_mut() {
                    mutator.set_render_opacity(value);
                }
            });
        }
        // A transparent shape usually defers its path, whose rebuild sends
        // Path dirt back here when shown. Skinned/follow-path shapes never
        // defer, so this paint must re-run any effects it skipped itself.
        if self.effects_deferred && value != 0.0 {
            self.base.add_dirt(ComponentDirt::PATH, false);
        }
    }

    pub fn paint(&self) -> Option<CoreHandle> {
        self.paint_mutator.clone()
    }

    pub fn mutator_is_translucent(&self) -> bool {
        self.paint_mutator
            .as_ref()
            .and_then(|mutator| {
                mutator.with(|mutator| {
                    mutator
                        .as_shape_paint_mutator()
                        .map(ShapePaintMutator::is_translucent)
                })
            })
            .flatten()
            .unwrap_or(false)
    }

    pub fn mutator_is_visible(&self) -> bool {
        self.paint_mutator
            .as_ref()
            .and_then(|mutator| {
                mutator.with(|mutator| {
                    mutator
                        .as_shape_paint_mutator()
                        .map(ShapePaintMutator::is_visible)
                })
            })
            .flatten()
            .unwrap_or(false)
    }

    pub fn parent_transform_component(&self) -> Option<CoreHandle> {
        let mut parent = self.base.parent_handle();
        while let Some(component) = parent {
            if component
                .with(|component| component.as_transform_component().is_some())
                .unwrap_or(false)
            {
                return Some(component);
            }
            parent = component
                .with(|component| component.component_parent_handle())
                .flatten();
        }
        None
    }
}

#[cfg(test)]
mod upstream_1db281b3_tests {
    use std::path::PathBuf;

    use nuxie_render_api::{PersistentFactory, RecordingFactory};

    use super::*;
    use crate::mechanical_port::source::{
        advance_flags::AdvanceFlags,
        artboard::Artboard,
        file::{File, ImportResult},
        generated::{
            core_registry::CoreRegistry, shapes::paint::feather_base::FeatherBase,
            text::text_modifier_group_base::TextModifierGroupBase,
        },
        text::{text::Text, text_modifier_group::TextModifierGroup},
    };

    fn fixture_path() -> PathBuf {
        PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets/text_feather_falloff.riv")
    }

    #[test]
    fn inner_feather_on_text_rebuilds_as_modifiers_change_the_glyphs() {
        let path = fixture_path();
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = crate::mechanical_port::source::factory::RuntimeFactoryHandle::from_factory(
            &mut factory,
        )
        .expect("retained factory");
        let mut result = ImportResult::Malformed;
        let file = File::import(&bytes, factory, Some(&mut result), None, None)
            .unwrap_or_else(|| panic!("fixture imports: {result:?}"));
        assert_eq!(result, ImportResult::Success);
        let artboard = file
            .with_file(File::artboard_default)
            .expect("default artboard");

        let feathers = artboard.with_artboard(|artboard| artboard.find_all_handles::<Feather>());
        let (feather, text) = feathers
            .into_iter()
            .find_map(|feather| {
                let text = feather
                    .with(|feather| feather.component_parent_handle())
                    .flatten()?
                    .with(|paint| paint.component_parent_handle())
                    .flatten()?
                    .with(|style| style.component_parent_handle())
                    .flatten()?;
                text.with_downcast::<Text, _>(Text::have_modifiers)
                    .filter(|has_modifiers| *has_modifiers)
                    .map(|_| (feather, text))
            })
            .expect("feathered fill on text with modifiers");
        let modifier_group = artboard
            .with_artboard(|artboard| artboard.find_all_handles::<TextModifierGroup>())
            .into_iter()
            .find(|group| {
                group
                    .with(|group| group.component_parent_handle().as_ref() == Some(&text))
                    .unwrap_or(false)
            })
            .expect("modifier group on feathered text");

        assert!(CoreRegistry::set_bool_handle(
            &feather,
            FeatherBase::INNER_PROPERTY_KEY.into(),
            true,
        ));
        assert!(
            feather
                .with_downcast::<Feather, _>(Feather::is_inner)
                .expect("Feather")
        );

        let advance = || {
            Artboard::advance_handle(
                &artboard.core_handle(),
                0.0,
                AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
            );
        };
        advance();
        let baseline = feather
            .with_downcast::<Feather, _>(|feather| feather.render_count)
            .expect("Feather");
        assert!(baseline > 0);

        let x = CoreRegistry::get_double_handle(
            &modifier_group,
            TextModifierGroupBase::X_PROPERTY_KEY.into(),
        )
        .expect("modifier x");
        assert!(CoreRegistry::set_double_handle(
            &modifier_group,
            TextModifierGroupBase::X_PROPERTY_KEY.into(),
            x + 10.0,
        ));
        advance();

        assert!(
            feather
                .with_downcast::<Feather, _>(|feather| feather.render_count)
                .expect("Feather")
                > baseline
        );
    }
}

impl std::ops::Deref for ShapePaint {
    type Target = ShapePaintBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ShapePaint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl EffectsContainer for ShapePaint {
    fn effects_state(&mut self) -> &mut EffectsContainerState {
        &mut self.effects_container
    }

    fn invalidate_effects(&mut self, effect: Option<&CoreHandle>) {
        self.invalidate_effects_from(effect);
    }

    fn add_stroke_effect(
        &mut self,
        identity: CoreHandle,
        effect: &mut dyn crate::mechanical_port::source::shapes::paint::stroke_effect::StrokeEffect,
    ) {
        ShapePaint::add_stroke_effect(self, identity, effect);
    }
}

#[cfg(test)]
#[path = "shape_paint_factory_contract_tests.rs"]
mod factory_contract_tests;
