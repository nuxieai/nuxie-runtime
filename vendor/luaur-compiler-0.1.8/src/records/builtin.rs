#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Builtin {
    pub(crate) object: luaur_ast::records::ast_name::AstName,
    pub(crate) method: luaur_ast::records::ast_name::AstName,
}
