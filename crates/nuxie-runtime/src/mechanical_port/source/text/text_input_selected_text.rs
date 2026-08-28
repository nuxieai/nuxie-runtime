use crate::mechanical_port::source::{
    core::CoreHandle, core_context::CoreContext,
    generated::text::text_input_selected_text_base::TextInputSelectedTextBase, math::mat2d::Mat2D,
    shapes::shape_paint_path::ShapePaintPath, status_code::StatusCode,
};
pub struct TextInputSelectedText {
    pub base: TextInputSelectedTextBase,
}
impl TextInputSelectedText {
    pub fn hit_test(&self, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.base
            .text_input_mut()
            .raw_text_input_mut()
            .separate_selection_text(true);
        StatusCode::Ok
    }
    pub fn local_clockwise_path(&mut self) -> Option<&mut ShapePaintPath> {
        Some(
            self.base
                .text_input_mut()
                .raw_text_input_mut()
                .selected_text_path(),
        )
    }
}
