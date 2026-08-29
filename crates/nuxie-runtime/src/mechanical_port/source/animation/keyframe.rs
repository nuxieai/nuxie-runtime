use crate::mechanical_port::source::{
    animation::keyed_property::KeyedProperty,
    generated::animation::keyframe_base::KeyFrameBase,
    importers::{import_stack::ImportStack, keyed_property_importer::KeyedPropertyImporter},
    status_code::StatusCode,
};

pub struct KeyFrame {
    pub base: KeyFrameBase,
    seconds: f32,
}

impl Default for KeyFrame {
    fn default() -> Self {
        Self {
            base: KeyFrameBase::default(),
            seconds: 0.0,
        }
    }
}

impl KeyFrame {
    pub fn seconds(&self) -> f32 {
        self.seconds
    }

    pub fn compute_seconds(&mut self, fps: i32) {
        self.seconds = self.base.frame() as f32 / fps as f32;
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<KeyedPropertyImporter>(crate::mechanical_port::source::generated::animation::keyed_property_base::KeyedPropertyBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_key_frame(this, self);
        self.base.base.import(import_stack)
    }
}
impl std::ops::Deref for KeyFrame {
    type Target = KeyFrameBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for KeyFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::keyframe_base::KeyFrameBaseCallbacks
    for KeyFrame
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
