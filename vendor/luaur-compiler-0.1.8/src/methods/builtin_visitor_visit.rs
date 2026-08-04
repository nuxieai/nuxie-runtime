use crate::records::symbol::Symbol;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_name::AstName;

pub fn extract_l_value_symbol(target: &AstExpr) -> Option<Symbol> {
    unsafe {
        let local = luaur_ast::rtti::ast_node_as::<AstExprLocal>(
            target as *const AstExpr as *mut luaur_ast::records::ast_node::AstNode,
        );
        if !local.is_null() {
            return Some(Symbol::from_local((*local).local));
        }

        let global = luaur_ast::rtti::ast_node_as::<AstExprGlobal>(
            target as *const AstExpr as *mut luaur_ast::records::ast_node::AstNode,
        );
        if !global.is_null() {
            return Some(Symbol::from_global((*global).name));
        }

        None
    }
}
