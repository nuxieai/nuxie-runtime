use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_stat_for::AstStatFor;
use luaur_ast::records::ast_type::AstType;

pub fn visit_ast_stat_for(this: &mut TypeMapVisitor<'_>, node: *mut AstStatFor) -> bool {
    unsafe {
        if !node.is_null() {
            let n = &*node;
            let ty = &this.builtin_types.number_type as *const _ as *const AstType;
            this.record_resolved_type_ast_local_ast_type(n.var, ty);
        }
    }
    true
}

impl TypeMapVisitor<'_> {
    pub fn visit_ast_stat_for(&mut self, node: *mut AstStatFor) -> bool {
        visit_ast_stat_for(self, node)
    }
}
