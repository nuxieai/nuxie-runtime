use crate::mechanical_port::source::{
    component::{Component, ComponentDirt, has_dirt},
    core::{CoreContext, StatusCode},
    math::mat2d::Mat2D,
    renderer::{RenderPaint, Renderer},
    shapes::{
        paint::{
            blend_mode::BlendMode,
            effects_container::EffectsContainer,
            feather::{Feather, TransformSpace},
            shape_paint_mutator::ShapePaintMutator,
            stroke_effect::StrokeEffect,
        },
        path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer,
        shape_paint_path::ShapePaintPath,
    },
    transform_component::TransformComponent,
};

pub struct ShapePaint {
    pub base: ShapePaintBase,
    pub effects_container: EffectsContainer,
    render_paint: Option<Box<RenderPaint>>,
    paint_mutator: Option<Box<dyn ShapePaintMutator>>,
    feather: Option<Feather>,
}

impl ShapePaint {
    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
        let Some(container) = ShapePaintContainer::from_component_mut(self.base.parent_mut())
        else {
            return StatusCode::MissingObject;
        };
        if self.paint_mutator.is_some() {
            container.add_paint(self);
        }
        StatusCode::Ok
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        if has_dirt(value, ComponentDirt::PATH) && !self.effects_container.effects().is_empty() {
            let container = ShapePaintContainer::from_component(self.base.parent()).unwrap();
            let mut path = self.pick_path(container);
            for effect in self.effects_container.effects_mut() {
                effect.update_effect(self, path, self);
                if let Some(new_path) = effect.effect_path(self) {
                    path = new_path;
                }
            }
        }
    }

    pub fn init_render_paint(&mut self, mutator: Box<dyn ShapePaintMutator>) -> &mut RenderPaint {
        assert!(self.render_paint.is_none());
        self.paint_mutator = Some(mutator);
        let factory = self
            .paint_mutator
            .as_ref()
            .unwrap()
            .component()
            .artboard()
            .factory();
        self.render_paint = Some(Box::new(factory.make_render_paint()));
        self.render_paint.as_deref_mut().unwrap()
    }

    pub fn blend_mode(&mut self, parent_value: BlendMode) {
        let render_paint = self.render_paint.as_deref_mut().unwrap();
        if self.base.blend_mode_value() == 127 {
            render_paint.set_blend_mode(parent_value);
        } else {
            render_paint.set_blend_mode(BlendMode::from_u8(self.base.blend_mode_value()));
        }
    }

    pub fn feather_mut(&mut self, feather: Feather) {
        self.feather = Some(feather);
    }

    pub fn feather(&self) -> Option<&Feather> {
        self.feather.as_ref()
    }

    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        shape_paint_path: &mut ShapePaintPath,
        transform: Mat2D,
        use_path_fill_rule: bool,
        mut override_paint: Option<&mut RenderPaint>,
        needs_save_operation: bool,
    ) {
        let mut path_to_draw = shape_paint_path;
        let mut saved = !needs_save_operation;
        if let Some(feather) = self.feather.as_ref() {
            let offset_in_artboard = feather.space() == TransformSpace::World;
            if offset_in_artboard
                && !feather.is_inner()
                && (feather.offset_x() != 0.0 || feather.offset_y() != 0.0)
            {
                if !saved {
                    saved = true;
                    renderer.save();
                }
                renderer.translate(feather.offset_x(), feather.offset_y());
            }
        }
        if shape_paint_path.is_local() {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(transform);
        }

        let path_effect = self.effects_container.last_effect_path(self);
        if let Some(path) = path_effect {
            path_to_draw = path;
        }

        if let Some(feather) = self.feather.as_mut() {
            if feather.is_inner() {
                let Some(inner_path) = feather.inner_path_mut() else {
                    return;
                };
                if path_effect.is_some() && feather.effect_path_dirty() {
                    if let Some(container) = ShapePaintContainer::from_component(self.base.parent())
                    {
                        feather.rebuild_inner_path(
                            path_effect.unwrap(),
                            container.shape_world_transform(),
                            feather.space() == TransformSpace::World,
                        );
                    }
                }
                path_to_draw = inner_path;
                if !saved {
                    saved = true;
                    renderer.save();
                }
                let clip_path = path_effect.unwrap_or(shape_paint_path);
                if let Some(render_path) = clip_path.render_path(self) {
                    renderer.clip_path(render_path);
                }
            }
            if feather.space() != TransformSpace::World
                && !feather.is_inner()
                && (feather.offset_x() != 0.0 || feather.offset_y() != 0.0)
            {
                if !saved {
                    saved = true;
                    renderer.save();
                }
                renderer.translate(feather.offset_x(), feather.offset_y());
            }
        }

        if let Some(render_path) = path_to_draw.render_path(self) {
            if !use_path_fill_rule {
                if let Some(fill) = self.as_fill() {
                    render_path.set_fill_rule(fill.fill_rule());
                }
            }
            let paint = override_paint
                .as_deref_mut()
                .unwrap_or_else(|| self.render_paint.as_deref_mut().unwrap());
            renderer.draw_path(render_path, paint);
        }
        if saved && needs_save_operation {
            renderer.restore();
        }
    }

    pub fn invalidate_effects_from(&mut self, effect: Option<&StrokeEffect>) {
        self.effects_container.invalidate_effects(effect);
        if let Some(feather) = self.feather.as_mut() {
            feather.mark_effect_path_dirty();
        }
        self.invalidate_rendering();
    }

    pub fn invalidate_effects(&mut self) {
        self.invalidate_effects_from(None);
    }

    pub fn invalidate_rendering(&mut self) {
        self.base.add_dirt(ComponentDirt::PATH);
    }

    pub fn add_stroke_effect(&mut self, mut effect: StrokeEffect) {
        effect.add_path_provider(self);
        self.effects_container.add_stroke_effect(effect);
    }

    pub fn render_opacity(&self) -> f32 {
        self.paint_mutator.as_ref().unwrap().render_opacity()
    }

    pub fn set_render_opacity(&mut self, value: f32) {
        self.paint_mutator
            .as_mut()
            .unwrap()
            .set_render_opacity(value);
    }

    pub fn is_flagged(&self, flags: PathFlags) -> bool {
        !(self.path_flags() & flags).is_empty()
    }

    pub fn render_paint(&mut self) -> &mut RenderPaint {
        self.render_paint.as_deref_mut().unwrap()
    }

    pub fn paint(&self) -> &Component {
        self.paint_mutator.as_ref().unwrap().component()
    }

    pub fn is_translucent(&self) -> bool {
        !self.base.is_visible() || self.paint_mutator.as_ref().unwrap().is_translucent()
    }

    pub fn should_draw(&self) -> bool {
        self.base.is_visible() && self.paint_mutator.as_ref().unwrap().is_visible()
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
