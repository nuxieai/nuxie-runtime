use crate::records::compiler::Compiler;
use crate::records::const_upvalue_visitor::ConstUpvalueVisitor;
use luaur_ast::records::ast_expr_function::AstExprFunction;

impl Compiler {
    pub fn gather_const_upvals(&mut self, func: *mut AstExprFunction) {
        let mut visitor = self.const_upvalue_visitor_const_upvalue_visitor();
        unsafe {
            luaur_ast::visit::ast_stat_block_visit(&*(*func).body, &mut visitor);
        }
        for local in visitor.upvals {
            self.get_upval(local);
        }
    }
}
