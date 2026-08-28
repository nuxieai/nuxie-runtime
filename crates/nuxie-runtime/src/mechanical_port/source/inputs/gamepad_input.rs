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

impl Default for GamepadInput {
    fn default() -> Self {
        Self {
            base: GamepadInputBase::default(),
        }
    }
}

impl GamepadInput {
    pub fn kind(&self) -> u32 {
        self.base.kind()
    }

    pub fn set_kind(&mut self, value: u32) {
        if self.base.set_kind_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(GamepadInputBase::KIND_PROPERTY_KEY);
        }
    }

    pub fn mapping(&self) -> u32 {
        self.base.mapping()
    }

    pub fn set_mapping(&mut self, value: u32) {
        if self.base.set_mapping_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(GamepadInputBase::MAPPING_PROPERTY_KEY);
        }
    }

    pub fn input_index(&self) -> u32 {
        self.base.input_index()
    }

    pub fn set_input_index(&mut self, value: u32) {
        if self.base.set_input_index_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(GamepadInputBase::INPUT_INDEX_PROPERTY_KEY);
        }
    }

    pub fn button_phase(&self) -> u32 {
        self.base.button_phase()
    }

    pub fn set_button_phase(&mut self, value: u32) {
        if self.base.set_button_phase_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(GamepadInputBase::BUTTON_PHASE_PROPERTY_KEY);
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        {
            let Some(lit_importer) = import_stack
                .latest::<ListenerInputTypeGamepadImporter>(ListenerInputTypeGamepadBase::TYPE_KEY)
            else {
                return StatusCode::MissingObject;
            };
            let listener_input_type = lit_importer.listener_input_type_gamepad();
            listener_input_type
                .with_downcast_mut::<
                    crate::mechanical_port::source::animation::listener_types::listener_input_type_gamepad::ListenerInputTypeGamepad,
                    _,
                >(|listener_input_type| listener_input_type.add_gamepad_input(this.clone()))
                .expect("ListenerInputTypeGamepadImporter retains its listener input type");
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
