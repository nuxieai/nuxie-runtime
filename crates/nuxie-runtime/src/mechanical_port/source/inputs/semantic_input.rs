use crate::mechanical_port::source::animation::listener_types::listener_input_type_semantic::ListenerInputTypeSemantic;
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

impl SemanticInput {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        {
            let Some(lit_importer) = import_stack.latest_mut::<ListenerInputTypeSemanticImporter>(
                ListenerInputTypeSemanticBase::TYPE_KEY,
            ) else {
                return StatusCode::MissingObject;
            };
            let listener_input_type: &mut ListenerInputTypeSemantic =
                lit_importer.listener_input_type_semantic_mut();
            listener_input_type.add_semantic_input(self);
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
