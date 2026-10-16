use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    generated::shapes::paint::angular_gradient_base::AngularGradientBase,
    math::vec2d::Vec2D,
    renderer::RenderPaint,
    shapes::paint::color::ColorInt,
    shapes::{
        paint::{
            linear_gradient::GradientKind,
            shape_paint_mutator::{ShapePaintMutator, ShapePaintMutatorState},
        },
        path_flags::PathFlags,
    },
};
#[derive(Default)]
pub struct AngularGradient {
    pub base: AngularGradientBase,
}
impl AngularGradient {
    pub fn make_gradient(
        &self,
        paint: &mut RenderPaint,
        start: Vec2D,
        end: Vec2D,
        colors: &[ColorInt],
        stops: &[f32],
    ) {
        self.base.base.make_gradient_with_kind(
            paint,
            start,
            end,
            colors,
            stops,
            GradientKind::Angular,
        );
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base
            .base
            .update_with_kind(value, GradientKind::Angular);
    }
}

impl ShapePaintMutator for AngularGradient {
    fn mutator_state(&self) -> &ShapePaintMutatorState {
        &self.base.base.mutator
    }
    fn mutator_state_mut(&mut self) -> &mut ShapePaintMutatorState {
        &mut self.base.base.mutator
    }
    fn render_opacity_changed(&mut self) {
        self.base.base.mark_gradient_dirty();
    }
    fn apply_to(&mut self, paint: &mut RenderPaint, opacity: f32, flags: PathFlags) {
        self.base.base.apply_to_with_kind(
            paint,
            opacity,
            GradientKind::Angular,
            !(flags & PathFlags::WORLD).is_empty(),
        );
    }
}
impl std::ops::Deref for AngularGradient {
    type Target = AngularGradientBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for AngularGradient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
