use crate::mechanical_port::source::{
    core::CoreHandle, generated::text::text_input_cursor_base::TextInputCursorBase,
    math::mat2d::Mat2D, shapes::shape_paint_path::ShapePaintPath,
};
pub struct TextInputCursor {
    pub base: TextInputCursorBase,
}
impl TextInputCursor {
    pub fn hit_test(&self, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }
    pub fn local_clockwise_path(&mut self) -> Option<&mut ShapePaintPath> {
        #[cfg(feature = "with_rive_text")]
        {
            if !self.base.text_input().is_focused() {
                return None;
            }
            return Some(
                self.base
                    .text_input_mut()
                    .raw_text_input_mut()
                    .cursor_path(),
            );
        }
        #[cfg(not(feature = "with_rive_text"))]
        {
            None
        }
    }
}
