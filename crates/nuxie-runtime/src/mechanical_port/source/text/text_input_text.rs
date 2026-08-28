use crate::mechanical_port::source::{
    core::CoreHandle, generated::text::text_input_text_base::TextInputTextBase, math::mat2d::Mat2D,
    shapes::shape_paint_path::ShapePaintPath,
};
pub struct TextInputText {
    pub base: TextInputTextBase,
}
impl TextInputText {
    pub fn hit_test(&self, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }
    pub fn local_clockwise_path(&mut self) -> Option<&mut ShapePaintPath> {
        Some(self.base.text_input_mut().raw_text_input_mut().text_path())
    }
}
