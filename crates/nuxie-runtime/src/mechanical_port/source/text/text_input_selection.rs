use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::text::text_input_selection_base::TextInputSelectionBase,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::paint::{shape_paint::ShapePaintPathKind, shape_paint_path::ShapePaintPath},
};
#[derive(Default)]
pub struct TextInputSelection {
    pub base: TextInputSelectionBase,
}
impl TextInputSelection {
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

                use_path(&mut parent.raw_text_input().selection_path().path);
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
            let path = Some(&mut parent.raw_text_input().selection_path().path);
            self.base.base.draw_with_path(renderer, path, world);
        });
    }
}
impl std::ops::Deref for TextInputSelection {
    type Target = TextInputSelectionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextInputSelection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
