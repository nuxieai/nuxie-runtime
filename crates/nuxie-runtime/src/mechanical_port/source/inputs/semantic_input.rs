use crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_semantic_base::ListenerInputTypeSemanticBase;
use crate::mechanical_port::source::generated::artboard_base::ArtboardBase;
use crate::mechanical_port::source::generated::inputs::semantic_input_base::SemanticInputBase;
use crate::mechanical_port::source::importers::artboard_importer::ArtboardImporter;
use crate::mechanical_port::source::importers::import_stack::ImportStack;
use crate::mechanical_port::source::importers::listener_input_type_semantic_importer::ListenerInputTypeSemanticImporter;
use crate::mechanical_port::source::status_code::StatusCode;

pub struct SemanticInput {
    pub base: SemanticInputBase,
}

impl Default for SemanticInput {
    fn default() -> Self {
        Self {
            base: SemanticInputBase::default(),
        }
    }
}

impl SemanticInput {
    pub fn action_type(&self) -> u32 {
        self.base.action_type()
    }

    pub fn set_action_type(&mut self, value: u32) {
        if self.base.set_action_type_value(value) {
            self.base
                .base
                .base
                .base
                .notify_property_changed(SemanticInputBase::ACTION_TYPE_PROPERTY_KEY);
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        {
            let Some(lit_importer) = import_stack.latest::<ListenerInputTypeSemanticImporter>(
                ListenerInputTypeSemanticBase::TYPE_KEY,
            ) else {
                return StatusCode::MissingObject;
            };
            let listener_input_type = lit_importer.listener_input_type_semantic();
            listener_input_type
                .with_downcast_mut::<
                    crate::mechanical_port::source::animation::listener_types::listener_input_type_semantic::ListenerInputTypeSemantic,
                    _,
                >(|listener_input_type| listener_input_type.add_semantic_input(this.clone()))
                .expect("ListenerInputTypeSemanticImporter retains its listener input type");
        }

        {
            let Some(artboard_importer) =
                import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
            else {
                return StatusCode::MissingObject;
            };
            artboard_importer.add_component(Some(this));
        }

        crate::mechanical_port::source::core::CoreObject::core_mut(self).import(import_stack)
    }
}
