use crate::mechanical_port::source::{
    component::{Component, ComponentDirt, has_dirt},
    core::{CoreContext, CoreHandle, StatusCode},
    math::mat2d::Mat2D,
    shapes::{
        paint::{
            blend_mode::BlendMode,
            effects_container::{EffectsContainer, EffectsContainerState},
            feather::Feather,
            shape_paint_mutator::ShapePaintMutator,
            stroke_effect::{PathProvider, StrokeEffect},
        },
        path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer,
        shape_paint_path::ShapePaintPath,
    },
    transform_component::TransformComponent,
    transform_space::TransformSpace,
};
use nuxie_render_api::{RenderPaint, Renderer};

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
    fn fill_rule(&self) -> Option<u32> {
        None
    }
    fn initialize_render_paint(&mut self, mutator: CoreHandle) -> bool;
    fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32);
}

pub struct ShapePaint {
    pub base: ShapePaintBase,
    pub effects_container: EffectsContainerState,
    path_provider: PathProvider,
    render_paint: Option<Box<dyn RenderPaint>>,
    paint_mutator: Option<CoreHandle>,
    feather: Option<CoreHandle>,
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
        }
    }
}

impl ShapePaint {
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        let Some(container) = ShapePaintContainer::from_component_mut(self.base.parent_mut())
        else {
            return StatusCode::MissingObject;
        };
        if self.paint_mutator.is_some() {
            if let Some(this) = self.base.handle() {
                container.add_paint(this);
            }
        }
        StatusCode::Ok
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        if has_dirt(value, ComponentDirt::PATH) && !self.effects_container.effects.is_empty() {
            let container = ShapePaintContainer::from_component(self.base.parent()).unwrap();
            let path = self.pick_path(container);
            let mut current = None;
            for effect in self.effects_container.effects.iter().cloned() {
                effect.with_mut(|effect| {
                    let Some(effect) = effect.as_stroke_effect_mut() else {
                        return;
                    };
                    if let Some(current) = current.as_ref() {
                        effect.update_effect(&self.path_provider, &current.borrow(), self);
                    } else {
                        effect.update_effect(&self.path_provider, path, self);
                    }
                    if let Some(new_path) = effect.effect_path(&self.path_provider) {
                        current = Some(new_path);
                    }
                });
            }
        }
    }

    pub fn init_render_paint(&mut self, mutator: CoreHandle) -> bool {
        if self.render_paint.is_some() {
            return false;
        }
        let Some(factory) = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
        else {
            return false;
        };
        self.paint_mutator = Some(mutator);
        self.render_paint = Some(factory.with_factory_mut(|factory| factory.make_render_paint()));
        true
    }

    pub fn with_render_paint_mut<R>(
        &mut self,
        use_paint: impl FnOnce(&mut dyn RenderPaint) -> R,
    ) -> Option<R> {
        self.render_paint.as_deref_mut().map(use_paint)
    }

    pub fn blend_mode(&mut self, parent_value: BlendMode) {
        let render_paint = self.render_paint.as_deref_mut().unwrap();
        if self.base.blend_mode_value() == 127 {
            render_paint.blend_mode(parent_value.into());
        } else {
            render_paint.blend_mode(BlendMode::from_u8(self.base.blend_mode_value()).into());
        }
    }

    pub fn set_feather(&mut self, feather: CoreHandle) {
        self.feather = Some(feather);
    }

    pub fn feather(&self) -> Option<CoreHandle> {
        self.feather.clone()
    }

    pub fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        shape_paint_path: &mut ShapePaintPath,
        transform: Mat2D,
        use_path_fill_rule: bool,
        override_paint: Option<&mut dyn RenderPaint>,
        needs_save_operation: bool,
    ) {
        let mut saved = !needs_save_operation;
        let feather = self.feather.as_ref().and_then(|feather| {
            feather.with_downcast::<Feather, _>(|feather| {
                (
                    feather.space(),
                    feather.is_inner(),
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
                        let transform = self
                            .base
                            .parent_handle()
                            .and_then(|parent| {
                                parent
                                    .with(|parent| {
                                        parent
                                            .as_shape_paint_container()
                                            .map(ShapePaintContainer::shape_world_transform)
                                    })
                                    .flatten()
                            })
                            .unwrap_or_else(Mat2D::identity);
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
                    let factory = self
                        .base
                        .with_artboard(|artboard| artboard.factory())
                        .flatten();
                    if let Some(factory) = factory {
                        if let Some(effect) = path_effect.as_ref() {
                            renderer.clip_path(effect.borrow_mut().render_path(&factory));
                        } else {
                            renderer.clip_path(shape_paint_path.render_path(&factory));
                        }
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

        let Some(factory) = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
        else {
            return;
        };
        let mut draw_path = |path: &mut ShapePaintPath| {
            let render_path = path.render_path(&factory);
            if !use_path_fill_rule {
                if let Some(fill_rule) = self.base.handle().and_then(|this| {
                    this.with(|this| {
                        this.as_shape_paint_behavior()
                            .and_then(ShapePaintBehavior::fill_rule)
                    })
                    .flatten()
                }) {
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
            } else if let Some(paint) = self.render_paint.as_deref() {
                renderer.draw_path(render_path, paint);
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
        let mut found = effect.is_none();
        for current in self.effects_container.effects.iter().cloned() {
            if found {
                current.with_mut(|current| {
                    if let Some(current) = current.as_stroke_effect_mut() {
                        current.invalidate_effect(None);
                    }
                });
            }
            if effect == Some(&current) {
                found = true;
            }
        }
        if let Some(feather) = self.feather.as_ref() {
            feather.with_downcast_mut::<Feather, _>(Feather::mark_effect_path_dirty);
        }
        self.invalidate_rendering();
    }

    pub fn invalidate_effects(&mut self) {
        self.invalidate_effects_from(None);
    }

    pub fn invalidate_rendering(&mut self) {
        self.base.add_dirt(ComponentDirt::PATH);
    }

    pub fn add_stroke_effect(&mut self, effect: CoreHandle) {
        effect.with_mut(|effect| {
            if let Some(effect) = effect.as_stroke_effect_mut() {
                effect.add_path_provider(&self.path_provider);
            }
        });
        EffectsContainer::add_stroke_effect(self, effect);
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

    pub fn is_flagged(&self, flags: PathFlags) -> bool {
        !(self.path_flags() & flags).is_empty()
    }

    pub fn paint(&self) -> Option<CoreHandle> {
        self.paint_mutator.clone()
    }

    pub fn is_translucent(&self) -> bool {
        !self.base.is_visible()
            || self
                .paint_mutator
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

    pub fn should_draw(&self) -> bool {
        self.base.is_visible()
            && self
                .paint_mutator
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

    pub fn parent_transform_component(&self) -> Option<&TransformComponent> {
        let mut parent = self.base.parent();
        while let Some(component) = parent {
            if let Some(transform) = component.as_transform_component() {
                return Some(transform);
            }
            parent = component.parent();
        }
        None
    }

    pub fn path_flags(&self) -> PathFlags {
        self.base.path_flags()
    }

    pub fn pick_path<'a>(&self, container: &'a ShapePaintContainer) -> &'a mut ShapePaintPath {
        self.base.pick_path(container)
    }
}

impl EffectsContainer for ShapePaint {
    fn effects_state(&mut self) -> &mut EffectsContainerState {
        &mut self.effects_container
    }

    fn invalidate_effects(&mut self, effect: Option<&CoreHandle>) {
        self.invalidate_effects_from(effect);
    }

    fn add_stroke_effect(&mut self, effect: CoreHandle) {
        ShapePaint::add_stroke_effect(self, effect);
    }
}
