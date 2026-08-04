use crate::records::ast_type::AstType;
use crate::records::name::Name;
use crate::records::position::Position;

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: Name,
    pub annotation: *mut AstType,
    pub colon_position: Position,
    pub is_const: bool,
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            name: unsafe { core::mem::zeroed() },
            annotation: core::ptr::null_mut(),
            colon_position: Position { line: 0, column: 0 },
            is_const: false,
        }
    }
}
