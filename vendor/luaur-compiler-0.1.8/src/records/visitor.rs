#[derive(Debug, Clone)]
pub struct Visitor {
    pub(crate) self_: *mut crate::records::compiler::Compiler,
    pub(crate) conflict: [u64; 4],
    pub(crate) assigned: [u64; 4],
}

impl luaur_ast::records::ast_visitor::AstVisitor for Visitor {
    fn visit_expr_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_local(node.cast())
    }
}
