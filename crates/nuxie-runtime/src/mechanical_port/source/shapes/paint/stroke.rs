pub use crate::mechanical_port::source::generated::shapes::paint::stroke_base::StrokeBase;

use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core::CoreHandle,
    shapes::{
        paint::{
            effects_container::{EffectsContainer, EffectsContainerState},
            shape_paint::{
                ShapePaint, ShapePaintBehavior, ShapePaintPathKind,
                ShapePaintRenderingInvalidation, ShapePaintType,
            },
            shape_paint_mutator::ShapePaintMutator,
            stroke_cap::StrokeCap,
            stroke_join::StrokeJoin,
        },
        path_flags::PathFlags,
    },
};
use nuxie_render_api::{RenderPaint, RenderPaintStyle, StrokePosition};

impl std::ops::Deref for Stroke {
    type Target = StrokeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Stroke {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Stroke {
    pub const TYPE_KEY: u16 = StrokeBase::TYPE_KEY;
}

#[derive(Default)]
pub struct Stroke {
    pub base: StrokeBase,
}

impl Stroke {
    pub fn path_flags(&self) -> PathFlags {
        transform_affects_stroke_path_flags(self.base.transform_affects_stroke())
    }

    pub fn init_render_paint(
        &mut self,
        mutator: CoreHandle,
        factory: &crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    ) -> bool {
        if !self.base.init_render_paint(mutator, factory) {
            return false;
        }
        let thickness = self.base.thickness();
        let cap: nuxie_render_api::StrokeCap = StrokeCap::from(u32::from(self.base.cap())).into();
        let join: nuxie_render_api::StrokeJoin =
            StrokeJoin::from(u32::from(self.base.join())).into();
        let position = self.stroke_position();
        self.base.with_render_paint_mut(|paint| {
            paint.style(RenderPaintStyle::Stroke);
            paint.thickness(thickness);
            paint.cap(cap);
            paint.join(join);
            paint.stroke_position(position);
        });
        true
    }

    pub fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32) {
        paint.style(RenderPaintStyle::Stroke);
        paint.thickness(self.base.thickness());
        paint.cap(StrokeCap::from(u32::from(self.base.cap())).into());
        paint.join(StrokeJoin::from(u32::from(self.base.join())).into());
        paint.stroke_position(self.stroke_position());
        paint.shader(None);
        let path_flags = self.path_flags();
        if let Some(mutator) = self.base.paint() {
            mutator.with_mut(|mutator| {
                if let Some(mutator) = mutator.as_shape_paint_mutator_mut() {
                    mutator.apply_to(paint, opacity, path_flags);
                }
            });
        }
    }

    pub fn is_visible(&self) -> bool {
        self.base.base.base.is_visible() && self.base.thickness() > 0.0
    }

    pub fn thickness_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }

    pub fn cap_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }

    pub fn join_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }

    pub fn position_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }

    pub fn stroke_position(&self) -> StrokePosition {
        match self.base.position() {
            0 => StrokePosition::Inside,
            2 => StrokePosition::Outside,
            _ => StrokePosition::Center,
        }
    }

    pub fn set_position(&mut self, value: u8) {
        if self.base.set_position_value(value) {
            self.position_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                StrokeBase::POSITION_PROPERTY_KEY,
            );
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        let kind = self.pick_path_kind();
        let stroke = self.effect_stroke();
        self.base.base.update_with_path_kind(value, kind, stroke);
        self.update_render_paint(value);
    }

    /// Pinned update at handle level, the component graph's handler: the
    /// path-effect pass runs with the stroke only read (see
    /// ShapePaint::update_effects_occurrence), then the render paint is
    /// updated under a new borrow, after it as before.
    pub(crate) fn update_occurrence(
        stroke: &crate::mechanical_port::source::core::CoreHandle,
        value: ComponentDirt,
    ) -> bool {
        let pass = stroke
            .with_downcast_mut::<Self, _>(|this| {
                let kind = this.pick_path_kind();
                let effect_stroke = this.effect_stroke();
                this.base.base.begin_effect_pass(value, kind, effect_stroke)
            })
            .flatten();
        if let Some(pass) = pass {
            crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaint::update_effects_occurrence(stroke, pass);
        }
        stroke.with_downcast_mut::<Self, _>(|stroke| stroke.update_render_paint(value));
        true
    }

    /// The thickness, cap and join a scripted effect's paint snapshot reads.
    fn effect_stroke(&self) -> Option<(f32, u32, u32)> {
        Some((
            self.base.thickness(),
            u32::from(self.base.cap()),
            u32::from(self.base.join()),
        ))
    }

    fn update_render_paint(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PAINT) {
            let thickness = self.base.thickness();
            let cap = StrokeCap::from(u32::from(self.base.cap())).into();
            let join = StrokeJoin::from(u32::from(self.base.join())).into();
            let position = self.stroke_position();
            self.base.with_render_paint_mut(|paint| {
                paint.thickness(thickness);
                paint.cap(cap);
                paint.join(join);
                paint.stroke_position(position);
            });
        }
    }

    pub fn invalidate_rendering(&mut self) {
        if self.prepare_rendering_invalidation().before_dirt() {
            self.base.invalidate_rendering_base();
        }
    }

    pub fn invalidate_effects_from(&mut self, effect: Option<&CoreHandle>) {
        ShapePaintBehavior::invalidate_effects_from(self, effect);
    }

    pub fn invalidate_effects(&mut self) {
        self.invalidate_effects_from(None);
    }

    pub fn build_dependencies(&mut self) {
        let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return;
        };
        let builder = parent
            .with(|parent| parent.shape_paint_path_builder())
            .flatten();
        if let Some(builder) = builder {
            builder.with_component_mut(|component| component.add_dependent(this));
        }
    }
}

impl ShapePaintBehavior for Stroke {
    fn prepare_rendering_invalidation(&self) -> ShapePaintRenderingInvalidation {
        ShapePaintRenderingInvalidation::stroke(self.base.render_paint_handle())
    }

    fn invalidate_rendering(&mut self) {
        Stroke::invalidate_rendering(self);
    }

    fn is_visible(&self) -> bool {
        Stroke::is_visible(self)
    }
    fn shape_paint(&self) -> &ShapePaint {
        &self.base.base
    }

    fn shape_paint_mut(&mut self) -> &mut ShapePaint {
        &mut self.base.base
    }

    fn path_flags(&self) -> PathFlags {
        Stroke::path_flags(self)
    }

    fn paint_type(&self) -> ShapePaintType {
        ShapePaintType::Stroke
    }

    fn pick_path_kind(&self) -> ShapePaintPathKind {
        if self.base.transform_affects_stroke() {
            ShapePaintPathKind::Local
        } else {
            ShapePaintPathKind::World
        }
    }

    fn initialize_render_paint(
        &mut self,
        mutator: CoreHandle,
        factory: &crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    ) -> bool {
        self.init_render_paint(mutator, factory)
    }

    fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32) {
        Stroke::apply_to(self, paint, opacity);
    }
}

impl EffectsContainer for Stroke {
    fn effects_state(&mut self) -> &mut EffectsContainerState {
        &mut self.base.base.effects_container
    }

    fn invalidate_effects(&mut self, effect: Option<&CoreHandle>) {
        ShapePaintBehavior::invalidate_effects_from(self, effect);
    }

    fn add_stroke_effect(
        &mut self,
        identity: CoreHandle,
        effect: &mut dyn crate::mechanical_port::source::shapes::paint::stroke_effect::StrokeEffect,
    ) {
        self.base.base.add_stroke_effect(identity, effect);
    }
}

/// Stroke::pathFlags() for a transformAffectsStroke value.
pub(crate) fn transform_affects_stroke_path_flags(transform_affects_stroke: bool) -> PathFlags {
    if transform_affects_stroke {
        PathFlags::LOCAL
    } else {
        PathFlags::WORLD
    }
}
