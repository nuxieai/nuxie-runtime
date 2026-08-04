use luaur_ast::records::ast_local::AstLocal;

#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) id: u32,
    pub(crate) upvals: Vec<*mut AstLocal>,
    pub(crate) cost_model: u64,
    pub(crate) stack_size: core::ffi::c_uint,
    pub(crate) can_inline: bool,
    pub(crate) returns_one: bool,
}

impl Default for Function {
    fn default() -> Self {
        Self {
            id: 0,
            upvals: Vec::new(),
            cost_model: 0,
            stack_size: 0,
            can_inline: false,
            returns_one: false,
        }
    }
}

impl luaur_common::records::dense_hash_table::DenseDefault for Function {
    fn dense_default() -> Self {
        Self::default()
    }
}
