use crate::records::value_visitor::ValueVisitor;
use luaur_ast::records::ast_stat_class::AstStatClass;

impl ValueVisitor {
    pub fn visit_ast_stat_class(&mut self, decl: *mut AstStatClass) -> bool {
        if !luaur_common::FFlag::DebugLuauUserDefinedClasses.get() {
            return false;
        }

        unsafe {
            let local = (*decl).name;
            *self.class_locals.get_or_insert((*local).name) = local;
            self.variables.get_or_insert(local).written = true;
        }

        true
    }
}
