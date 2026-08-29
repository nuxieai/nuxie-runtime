use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_input_selected_text_base::TextInputSelectedTextBase,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::paint::{shape_paint::ShapePaintPathKind, shape_paint_path::ShapePaintPath},
    status_code::StatusCode,
};
#[derive(Default)]
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
        self.base.text_input_handle().with_mut(|parent| {
            parent
                .as_text_input_mut()
                .expect("TextInputSelectedText parent")
                .raw_text_input()
                .set_separate_selection_text(true);
        });
        StatusCode::Ok
    }
    pub fn with_path_mut(
        &self,
        kind: ShapePaintPathKind,
        use_path: &mut dyn FnMut(&mut ShapePaintPath),
    ) -> bool {
        if kind == ShapePaintPathKind::World {
            unreachable!("TextInputDrawable::worldPath is unreachable upstream");
        }
        self.base
            .text_input_handle()
            .with_mut(|parent| {
                let parent = parent
                    .as_text_input_mut()
                    .expect("TextInputDrawable parent");

                use_path(parent.raw_text_input().selected_text_path());
                true
            })
            .unwrap_or(false)
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        self.base.text_input_handle().with_mut(|parent| {
            let parent = parent
                .as_text_input_mut()
                .expect("TextInputDrawable parent");
            let world = *parent.base.world_transform();
            let path = Some(parent.raw_text_input().selected_text_path());
            self.base.base.draw_with_path(renderer, path, world);
        });
    }
}
impl std::ops::Deref for TextInputSelectedText {
    type Target = TextInputSelectedTextBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextInputSelectedText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
