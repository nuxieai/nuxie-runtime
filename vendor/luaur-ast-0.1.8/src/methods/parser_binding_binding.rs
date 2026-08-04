use crate::records::ast_type::AstType;
use crate::records::binding::Binding;
use crate::records::name::Name;
use crate::records::position::Position;

impl Binding {
    pub fn new(
        name: Name,
        annotation: *mut AstType,
        colon_position: Position,
        is_const: bool,
    ) -> Self {
        Self {
            name,
            annotation,
            colon_position,
            is_const,
        }
    }
}

#[allow(non_snake_case)]
pub fn parser_binding_binding(
    name: Name,
    annotation: *mut AstType,
    colon_position: Position,
    is_const: bool,
) -> Binding {
    Binding::new(name, annotation, colon_position, is_const)
}
