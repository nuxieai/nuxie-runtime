//! Shared child operations from `rive/scripted/transition_child_ref.hpp`.

use crate::mechanical_port::source::{
    artboard::Artboard, renderer::Renderer, scripted::script_backend::TransitionChildRef,
};

impl TransitionChildRef {
    /// Draw internal content even when the child is hidden from the normal loop.
    pub fn draw(&self, renderer: &mut Renderer) {
        let Some(artboard) = &self.artboard else {
            return;
        };
        renderer.save();
        renderer.transform(self.transform);
        Artboard::draw_internal_handle(artboard, renderer);
        renderer.restore();
    }

    pub fn width(&self) -> f32 {
        self.artboard
            .as_ref()
            .and_then(|artboard| artboard.with_downcast::<Artboard, _>(Artboard::width))
            .unwrap_or(0.0)
    }

    pub fn height(&self) -> f32 {
        self.artboard
            .as_ref()
            .and_then(|artboard| artboard.with_downcast::<Artboard, _>(Artboard::height))
            .unwrap_or(0.0)
    }
}
