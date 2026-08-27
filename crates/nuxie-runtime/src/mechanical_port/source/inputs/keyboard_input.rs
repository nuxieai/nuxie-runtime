use crate::mechanical_port::source::animation::listener_types::listener_input_type_keyboard::ListenerInputTypeKeyboard;
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

impl KeyboardInput {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        {
            let Some(lit_importer) = import_stack.latest_mut::<ListenerInputTypeKeyboardImporter>(
                ListenerInputTypeKeyboardBase::TYPE_KEY,
            ) else {
                return StatusCode::MissingObject;
            };
            let listener_input_type: &mut ListenerInputTypeKeyboard =
                lit_importer.listener_input_type_keyboard_mut();
            listener_input_type.add_keyboard_input(self);
        }

        {
            let Some(artboard_importer) =
                import_stack.latest_mut::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
            else {
                return StatusCode::MissingObject;
            };
            artboard_importer.add_component(self);
        }

        self.base.import(import_stack)
    }
}
