use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_function::AstStatFunction;
use luaur_ast::records::ast_visitor::AstVisitor;

impl dyn AstVisitor {
    #[allow(non_snake_case)]
    pub fn visit_ast_stat_function(&mut self, node: *mut AstStatFunction) -> bool {
        self.visit_stat(node as *mut AstStat as *mut core::ffi::c_void)
    }
}
