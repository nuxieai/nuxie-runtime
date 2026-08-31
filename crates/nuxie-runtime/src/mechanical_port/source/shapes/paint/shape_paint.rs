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
    path_provider: PathProvider,
    render_paint: Option<RuntimeRenderPaintHandle>,
    paint_mutator: Option<CoreHandle>,
    feather: Option<CoreHandle>,
    script_paint_scope: Option<Rc<crate::scripting::ScriptPaint>>,
}

impl Default for ShapePaint {
    fn default() -> Self {
        Self {
            base: ShapePaintBase::default(),
            effects_container: EffectsContainerState::default(),
            path_provider: PathProvider::default(),
            render_paint: None,
            paint_mutator: None,
            feather: None,
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
        use crate::mechanical_port::source::generated::shapes::paint::{
            fill_base::FillBase, stroke_base::StrokeBase,
        };
        // The cached type is immutable, so effects can inspect their active
        // parent paint without reborrowing the Fill/Stroke occurrence.
        match self
            .base
            .handle()
            .expect("installed ShapePaint occurrence")
            .core_type()
        {
            Some(FillBase::TYPE_KEY) => ShapePaintType::Fill,
            Some(StrokeBase::TYPE_KEY) => ShapePaintType::Stroke,
            type_key => panic!("abstract ShapePaint has no paint type: {type_key:?}"),
        }
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
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

    pub fn blend_mode(&mut self, parent_value: BlendMode) {
        let mut render_paint = self.render_paint.as_ref().unwrap().borrow_mut();
        if self.base.blend_mode_value() == 127 {
            render_paint.blend_mode(parent_value);
        } else {
            let mode = match self.base.blend_mode_value() {
                3 => BlendMode::SrcOver,
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
            };
            render_paint.blend_mode(mode);
        }
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
        let Some(factory) = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
        else {
            return;
        };
        self.draw_with_factory(
            renderer,
            shape_paint_path,
            transform,
            use_path_fill_rule,
            override_paint,
            needs_save_operation,
            fill_rule,
            &factory,
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
        factory: &crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    ) {
        let mut saved = !needs_save_operation;
        let feather = self.feather.as_ref().and_then(|feather| {
            feather.with_downcast::<Feather, _>(|feather| {
                (
                    feather.space(),
                    feather.base.inner() && fill_rule.is_some(),
                    feather.base.offset_x(),
                    feather.base.offset_y(),
                    feather.effect_path_dirty(),
                    feather.inner_path(),
                )
            })
        });
        if let Some((space, is_inner, offset_x, offset_y, _, _)) = feather.as_ref() {
            if *space == TransformSpace::World
                && !*is_inner
                && (*offset_x != 0.0 || *offset_y != 0.0)
            {
                if !saved {
                    saved = true;
                    renderer.save();
                }
                renderer.translate(*offset_x, *offset_y);
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
        let inner_path =
            feather
                .as_ref()
                .and_then(|(space, is_inner, _, _, effect_path_dirty, inner_path)| {
                    if !*is_inner {
                        return None;
                    }
                    if path_effect.is_some() && *effect_path_dirty {
                        if let (Some(feather), Some(effect)) =
                            (self.feather.as_ref(), path_effect.as_ref())
                        {
                            feather.with_downcast_mut::<Feather, _>(|feather| {
                                feather.rebuild_inner_path(
                                    &effect.borrow(),
                                    &transform,
                                    *space == TransformSpace::World,
                                );
                            });
                        }
                    }
                    if !saved {
                        saved = true;
                        renderer.save();
                    }
                    if let Some(effect) = path_effect.as_ref() {
                        renderer.clip_path(effect.borrow_mut().render_path(factory));
                    } else {
                        renderer.clip_path(shape_paint_path.render_path(factory));
                    }
                    Some(inner_path.clone())
                });
        if let Some((space, is_inner, offset_x, offset_y, _, _)) = feather.as_ref() {
            if *space != TransformSpace::World
                && !*is_inner
                && (*offset_x != 0.0 || *offset_y != 0.0)
            {
                if !saved {
                    saved = true;
                    renderer.save();
                }
                renderer.translate(*offset_x, *offset_y);
            }
        }

        let mut draw_path = |path: &mut ShapePaintPath| {
            let render_path = path.render_path(factory);
            if !use_path_fill_rule {
                if let Some(fill_rule) = fill_rule {
                    match fill_rule {
                        0 => render_path.fill_rule(nuxie_render_api::FillRule::NonZero),
                        1 => render_path.fill_rule(nuxie_render_api::FillRule::EvenOdd),
                        2 => render_path.fill_rule(nuxie_render_api::FillRule::Clockwise),
                        _ => {}
                    }
                }
            }
            if let Some(paint) = override_paint.as_deref() {
                renderer.draw_path(render_path, paint);
            } else if let Some(paint) = self.render_paint.as_ref() {
                renderer.draw_path(render_path, paint.borrow().as_ref());
            }
        };
        if let Some(inner) = inner_path {
            draw_path(&mut inner.borrow_mut());
        } else if let Some(effect) = path_effect {
            draw_path(&mut effect.borrow_mut());
        } else {
            draw_path(shape_paint_path);
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
            feather.with_downcast_mut::<Feather, _>(Feather::mark_effect_path_dirty);
        }
    }

    pub fn invalidate_effects(&mut self) {
        self.invalidate_effects_from(None);
    }

    pub fn invalidate_rendering(&mut self) {
        self.base.add_dirt(ComponentDirt::PATH, true);
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
