use crate::mechanical_port::source::{
    core::CoreHandle, generated::text::text_input_selection_base::TextInputSelectionBase,
    math::mat2d::Mat2D, shapes::shape_paint_path::ShapePaintPath,
};
pub struct TextInputSelection {
    pub base: TextInputSelectionBase,
}
impl TextInputSelection {
    pub fn hit_test(&self, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }
    pub fn local_clockwise_path(&mut self) -> Option<&mut ShapePaintPath> {
        #[cfg(feature = "rive_text")]
        {
            return Some(
                self.base
                    .text_input_mut()
                    .raw_text_input_mut()
                    .selection_path(),
            );
        }
        #[cfg(not(feature = "rive_text"))]
        {
            None
        }
    }
}
