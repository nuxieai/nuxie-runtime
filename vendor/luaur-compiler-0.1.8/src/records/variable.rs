use luaur_ast::records::ast_expr::AstExpr;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Variable {
    pub(crate) init: *mut AstExpr, // initial value of the variable; filled by trackValues
    pub(crate) written: bool,      // is the variable ever assigned to? filled by trackValues
    pub(crate) constant: bool, // is the variable's value a compile-time constant? filled by constantFold
}

impl luaur_common::records::dense_hash_table::DenseDefault for Variable {
    fn dense_default() -> Self {
        Self::default()
    }
}

impl Default for Variable {
    fn default() -> Self {
        Self {
            init: core::ptr::null_mut(),
            written: false,
            constant: false,
        }
    }
}
