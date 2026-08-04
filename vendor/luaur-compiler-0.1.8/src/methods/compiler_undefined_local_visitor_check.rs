use crate::records::undefined_local_visitor::UndefinedLocalVisitor;
use luaur_ast::records::ast_local::AstLocal;

impl UndefinedLocalVisitor {
    pub fn check(&mut self, local: *mut AstLocal) {
        if self.undef.is_null() && self.locals.contains(&local) {
            self.undef = local;
        }
    }
}
