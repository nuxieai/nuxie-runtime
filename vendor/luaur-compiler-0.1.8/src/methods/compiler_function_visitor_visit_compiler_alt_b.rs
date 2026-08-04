use crate::records::function_visitor::FunctionVisitor;
use luaur_ast::records::ast_stat_type_function::AstStatTypeFunction;

impl<'a> FunctionVisitor<'a> {
    pub fn visit_ast_stat_type_function(&mut self, _node: *mut AstStatTypeFunction) -> bool {
        false
    }
}
