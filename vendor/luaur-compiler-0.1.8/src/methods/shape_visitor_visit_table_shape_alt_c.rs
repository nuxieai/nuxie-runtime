use luaur_ast::records::ast_stat_function::AstStatFunction;
use crate::records::shape_visitor::ShapeVisitor;

impl<'a> ShapeVisitor<'a> {
    pub fn visit_ast_stat_function(&mut self, node: *mut AstStatFunction) -> bool {
        unsafe {
            let node = &*node;
            self.assign(node.name);
            self.visit_ast_stat_local(node.func as *mut _);
        }

        false
    }
}
