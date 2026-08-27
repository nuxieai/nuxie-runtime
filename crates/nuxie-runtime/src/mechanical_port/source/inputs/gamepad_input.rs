use crate::mechanical_port::source::animation::listener_types::listener_input_type_gamepad::ListenerInputTypeGamepad;
use crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_gamepad_base::ListenerInputTypeGamepadBase;
use crate::mechanical_port::source::generated::artboard_base::ArtboardBase;
use crate::mechanical_port::source::generated::inputs::gamepad_input_base::GamepadInputBase;
use crate::mechanical_port::source::importers::artboard_importer::ArtboardImporter;
use crate::mechanical_port::source::importers::import_stack::ImportStack;
use crate::mechanical_port::source::importers::listener_input_type_gamepad_importer::ListenerInputTypeGamepadImporter;
use crate::mechanical_port::source::status_code::StatusCode;

pub struct GamepadInput {
    pub base: GamepadInputBase,
}

impl GamepadInput {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        {
            let Some(lit_importer) = import_stack.latest_mut::<ListenerInputTypeGamepadImporter>(
                ListenerInputTypeGamepadBase::TYPE_KEY,
            ) else {
                return StatusCode::MissingObject;
            };
            let listener_input_type: &mut ListenerInputTypeGamepad =
                lit_importer.listener_input_type_gamepad_mut();
            listener_input_type.add_gamepad_input(self);
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
