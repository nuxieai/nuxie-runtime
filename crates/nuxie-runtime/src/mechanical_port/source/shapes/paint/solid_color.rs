use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::solid_color_base::SolidColorBase,
    renderer::RenderPaint,
    shapes::paint::{
        color::{color_modulate_opacity, color_opacity},
        shape_paint_mutator::{MutatorFlags, ShapePaintMutator, ShapePaintMutatorState},
    },
};
#[derive(Default)]
pub struct SolidColor {
    pub base: SolidColorBase,
    mutator: ShapePaintMutatorState,
}
impl SolidColor {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let mut code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        code = self.init_paint_mutator(this, self.base.parent_handle());
        if code == StatusCode::Ok {
            self.render_opacity_changed();
        }
        code
    }
    pub fn render_opacity_changed(&mut self) {
        let Some(paint) = self.mutator.render_paint_handle() else {
            return;
        };
        let value = color_modulate_opacity(self.base.color_value() as u32, self.render_opacity());
        paint.borrow_mut().color(value);
        let opacity = color_opacity(value);
        self.mutator.flags = MutatorFlags::NONE;
        if opacity > 0.0 {
            self.mutator.flags |= MutatorFlags::VISIBLE;
        } else if opacity < 1.0 {
            self.mutator.flags |= MutatorFlags::TRANSLUCENT;
        }
        self.base.with_artboard_mut(|artboard| artboard.changed());
    }
    pub fn apply_to(&self, paint: &mut RenderPaint, opacity_modifier: f32) {
        paint.color(color_modulate_opacity(
            self.base.color_value() as u32,
            self.render_opacity() * opacity_modifier,
        ));
    }
    pub fn color_value_changed(&mut self) {
        self.render_opacity_changed();
    }
}

impl ShapePaintMutator for SolidColor {
    fn mutator_state(&self) -> &ShapePaintMutatorState {
        &self.mutator
    }
    fn mutator_state_mut(&mut self) -> &mut ShapePaintMutatorState {
        &mut self.mutator
    }
    fn render_opacity_changed(&mut self) {
        SolidColor::render_opacity_changed(self);
    }
    fn apply_to(
        &mut self,
        paint: &mut RenderPaint,
        opacity: f32,
        _: crate::mechanical_port::source::shapes::path_flags::PathFlags,
    ) {
        SolidColor::apply_to(self, paint, opacity);
    }
}
impl std::ops::Deref for SolidColor {
    type Target = SolidColorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for SolidColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
