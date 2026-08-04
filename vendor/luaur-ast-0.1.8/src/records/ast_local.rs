use crate::records::ast_name::AstName;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstLocal {
    pub name: AstName,
    pub location: Location,
    pub shadow: *mut AstLocal,
    pub function_depth: usize,
    pub loop_depth: usize,
    pub is_const: bool,
    pub is_exported: bool,
    pub annotation: *mut AstType,
}

// NB: `AstLocal` is a plain arena struct, NOT an `AstNode` subclass (no
// `LUAU_RTTI` in C++) — it carries no class index and is not part of the visitor
// dispatch.
