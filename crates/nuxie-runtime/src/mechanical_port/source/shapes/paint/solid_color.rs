use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::solid_color_base::SolidColorBase,
    renderer::RenderPaint,
    shapes::paint::{
        color::{color_modulate_opacity, color_opacity},
        shape_paint_mutator::{MutatorFlags, ShapePaintMutator},
    },
};
pub struct SolidColor {
    pub base: SolidColorBase,
    pub flags: MutatorFlags,
}
impl SolidColor {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let mut code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        code = self.base.init_paint_mutator(self);
        if code == StatusCode::Ok {
            self.render_opacity_changed();
        }
        code
    }
    pub fn render_opacity_changed(&mut self) {
        let Some(paint) = self.base.render_paint_mut() else {
            return;
        };
        let value = color_modulate_opacity(self.base.color_value(), self.base.render_opacity());
        paint.set_color(value);
        let opacity = color_opacity(value);
        self.flags = MutatorFlags::NONE;
        if opacity > 0.0 {
            self.flags |= MutatorFlags::VISIBLE;
        } else if opacity < 1.0 {
            self.flags |= MutatorFlags::TRANSLUCENT;
        }
        if let Some(artboard) = self.base.artboard_mut() {
            artboard.changed();
        }
    }
    pub fn apply_to(&self, paint: &mut RenderPaint, opacity_modifier: f32) {
        paint.set_color(color_modulate_opacity(
            self.base.color_value(),
            self.base.render_opacity() * opacity_modifier,
        ));
    }
    pub fn color_value_changed(&mut self) {
        self.render_opacity_changed();
    }
}
