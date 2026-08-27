use std::{ptr::NonNull, sync::OnceLock};

use crate::mechanical_port::source::{
    animation::linear_animation::LinearAnimation,
    generated::{
        animation::{blend_animation_base::BlendAnimationBase, layer_state_base::LayerStateBase},
        artboard_base::ArtboardBase,
    },
    importers::{
        artboard_importer::ArtboardImporter, import_stack::ImportStack,
        layer_state_importer::LayerStateImporter,
    },
    status_code::StatusCode,
};

static EMPTY_ANIMATION: OnceLock<LinearAnimation> = OnceLock::new();

pub struct BlendAnimation {
    pub base: BlendAnimationBase,
    animation: Option<NonNull<LinearAnimation>>,
}

impl Default for BlendAnimation {
    fn default() -> Self {
        Self {
            base: BlendAnimationBase::default(),
            animation: None,
        }
    }
}

impl BlendAnimation {
    pub fn animation(&self) -> &LinearAnimation {
        self.animation
            .map(|animation| unsafe { animation.as_ref() })
            .unwrap_or_else(|| EMPTY_ANIMATION.get_or_init(LinearAnimation::default))
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<LayerStateImporter>(LayerStateBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        if !importer.add_blend_animation(NonNull::from(&mut *self)) {
            return StatusCode::InvalidObject;
        }

        let Some(artboard_importer) =
            import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let artboard = artboard_importer.artboard();
        let animation_id = self.base.animation_id() as usize;
        unsafe {
            if animation_id < artboard.as_ref().animation_count() {
                self.animation = artboard.as_ref().animation(animation_id);
            }
        }

        StatusCode::Ok
    }
}
