use crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_keyboard_base::ListenerInputTypeKeyboardBase;
use crate::mechanical_port::source::generated::artboard_base::ArtboardBase;
use crate::mechanical_port::source::generated::inputs::keyboard_input_base::KeyboardInputBase;
use crate::mechanical_port::source::importers::artboard_importer::ArtboardImporter;
use crate::mechanical_port::source::importers::import_stack::ImportStack;
use crate::mechanical_port::source::importers::listener_input_type_keyboard_importer::ListenerInputTypeKeyboardImporter;
use crate::mechanical_port::source::status_code::StatusCode;

pub struct KeyboardInput {
    pub base: KeyboardInputBase,
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self {
            base: KeyboardInputBase::default(),
        }
    }
}

impl KeyboardInput {
    pub fn key_type(&self) -> u32 {
        self.base.key_type()
    }

    pub fn set_key_type(&mut self, value: u32) {
        if self.base.set_key_type_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(KeyboardInputBase::KEY_TYPE_PROPERTY_KEY);
        }
    }

    pub fn key_phase(&self) -> u32 {
        self.base.key_phase()
    }

    pub fn set_key_phase(&mut self, value: u32) {
        if self.base.set_key_phase_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(KeyboardInputBase::KEY_PHASE_PROPERTY_KEY);
        }
    }

    pub fn modifiers(&self) -> u32 {
        self.base.modifiers()
    }

    pub fn set_modifiers(&mut self, value: u32) {
        if self.base.set_modifiers_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(KeyboardInputBase::MODIFIERS_PROPERTY_KEY);
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        {
            let Some(lit_importer) = import_stack.latest::<ListenerInputTypeKeyboardImporter>(
                ListenerInputTypeKeyboardBase::TYPE_KEY,
            ) else {
                return StatusCode::MissingObject;
            };
            let listener_input_type = lit_importer.listener_input_type_keyboard();
            listener_input_type
                .with_downcast_mut::<
                    crate::mechanical_port::source::animation::listener_types::listener_input_type_keyboard::ListenerInputTypeKeyboard,
                    _,
                >(|listener_input_type| listener_input_type.add_keyboard_input(this.clone()))
                .expect("ListenerInputTypeKeyboardImporter retains its listener input type");
        }

        {
            let Some(artboard_importer) =
                import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
            else {
                return StatusCode::MissingObject;
            };
            artboard_importer.add_component(Some(this));
        }

        self.base.import(import_stack)
    }
}
