use crate::mechanical_port::source::{
    generated::shapes::paint::radial_gradient_base::RadialGradientBase, math::vec2d::Vec2D,
    renderer::RenderPaint, shapes::paint::color::ColorInt,
};
pub struct RadialGradient {
    pub base: RadialGradientBase,
}
impl RadialGradient {
    pub fn make_gradient(
        &self,
        paint: &mut RenderPaint,
        start: Vec2D,
        end: Vec2D,
        colors: &[ColorInt],
        stops: &[f32],
    ) {
        let factory = self
            .base
            .artboard()
            .factory()
            .expect("RadialGradient requires its Artboard renderer factory");
        let shader = factory.with_factory_mut(|factory| {
            factory.make_radial_gradient(
                start.x,
                start.y,
                Vec2D::distance(start, end),
                colors,
                stops,
            )
        });
        paint.shader(Some(shader.as_ref()));
    }
}
