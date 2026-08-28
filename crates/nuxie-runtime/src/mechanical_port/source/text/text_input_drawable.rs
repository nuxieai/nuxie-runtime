use crate::mechanical_port::source::{
    artboard::Artboard,
    core_context::CoreContext,
    generated::text::text_input_drawable_base::TextInputDrawableBase,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::{shape_paint_container::ShapePaintContainer, shape_paint_path::ShapePaintPath},
    status_code::StatusCode,
    text::text_input::TextInput,
};
pub struct TextInputDrawable {
    pub base: TextInputDrawableBase,
    pub paints: ShapePaintContainer,
}
impl TextInputDrawable {
    fn get_artboard(&self) -> &Artboard {
        self.base.artboard()
    }
    pub fn text_input(&self) -> &TextInput {
        self.base
            .parent_as_text_input()
            .expect("TextInputDrawable TextInput parent")
    }
    pub fn world_path(&mut self) -> &mut ShapePaintPath {
        unreachable!("TextInputDrawable::worldPath is unreachable upstream")
    }
    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        self.base.local_clockwise_path()
    }
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        if !self.base.parent_is_text_input() {
            StatusCode::InvalidObject
        } else {
            StatusCode::Ok
        }
    }
    pub fn shape_world_transform(&self) -> &Mat2D {
        self.base.world_transform()
    }
    pub fn path_builder(&mut self) -> crate::mechanical_port::source::core::CoreHandle {
        self.base.parent().expect("TextInputDrawable parent")
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        for paint in self.paints.shape_paints_mut() {
            if !paint.is_visible() {
                continue;
            }
            let Some(path) = paint.pick_path(&mut self.base) else {
                continue;
            };
            paint.draw(renderer, path, self.text_input().base.world_transform());
        }
    }
    pub fn will_draw(&self) -> bool {
        self.base.will_draw() && self.base.render_opacity() != 0.0
    }
}
