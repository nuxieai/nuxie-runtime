use crate::mechanical_port::source::generated::inputs::user_input_base::UserInputBase;

#[derive(Default)]
pub struct UserInput {
    pub base: UserInputBase,
}

impl UserInput {
    pub fn claims_artboard_slot(
        _import_stack: &mut crate::source::importers::import_stack::ImportStack,
    ) -> bool {
        true
    }
}
