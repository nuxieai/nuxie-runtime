use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::text::text_input_cursor_base::TextInputCursorBase,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::paint::{shape_paint::ShapePaintPathKind, shape_paint_path::ShapePaintPath},
};
#[derive(Default)]
pub struct TextInputCursor {
    pub base: TextInputCursorBase,
}
impl TextInputCursor {
    pub fn hit_test(&self, _transform: &Mat2D) -> Option<CoreHandle> {
        None
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
                if !parent.is_focused() {
                    return false;
                }
                use_path(parent.raw_text_input().cursor_path());
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
            let path = if parent.is_focused() {
                Some(parent.raw_text_input().cursor_path())
            } else {
                None
            };
            self.base.base.draw_with_path(renderer, path, world);
        });
    }
}
impl std::ops::Deref for TextInputCursor {
    type Target = TextInputCursorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextInputCursor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
