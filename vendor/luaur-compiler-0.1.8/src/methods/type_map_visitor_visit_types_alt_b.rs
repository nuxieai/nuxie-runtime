use crate::methods::type_map_visitor_push_type_aliases::type_map_visitor_push_type_aliases;
use crate::records::type_map_visitor::TypeMapVisitor;

use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_repeat::AstStatRepeat;

pub fn visit_ast_stat_repeat(this: &mut TypeMapVisitor<'_>, node: *mut AstStatRepeat) -> bool {
    unsafe {
        if node.is_null() {
            return false;
        }

        let repeat = &mut *node;

        let alias_stack_top = type_map_visitor_push_type_aliases(this, repeat.body);

        for stat_ptr in (*repeat.body).body.as_slice() {
            let stat: &mut AstStat = &mut **stat_ptr;
            luaur_ast::visit::ast_stat_visit(stat, this);
        }

        if !repeat.condition.is_null() {
            luaur_ast::visit::ast_expr_visit(repeat.condition, this);
        }

        this.pop_type_aliases(alias_stack_top);
    }

    false
}

impl TypeMapVisitor<'_> {
    pub fn visit_ast_stat_repeat(&mut self, node: *mut AstStatRepeat) -> bool {
        visit_ast_stat_repeat(self, node)
    }
}
