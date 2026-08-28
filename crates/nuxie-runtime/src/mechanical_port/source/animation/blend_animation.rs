use crate::mechanical_port::source::{
    core::CoreHandle,
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

pub struct BlendAnimation {
    pub base: BlendAnimationBase,
    animation: Option<CoreHandle>,
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
    pub fn animation(&self) -> Option<CoreHandle> {
        self.animation.clone()
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<LayerStateImporter>(LayerStateBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        if !importer.add_blend_animation(this) {
            return StatusCode::InvalidObject;
        }

        let Some(artboard_importer) =
            import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let artboard = artboard_importer.artboard();
        let animation_id = self.base.animation_id() as usize;
        self.animation = artboard
            .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(|artboard| {
                artboard.animation_handle_at(animation_id)
            })
            .flatten();

        StatusCode::Ok
    }
}
impl std::ops::Deref for BlendAnimation {
    type Target = BlendAnimationBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BlendAnimation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::blend_animation_base::BlendAnimationBaseCallbacks for BlendAnimation { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
