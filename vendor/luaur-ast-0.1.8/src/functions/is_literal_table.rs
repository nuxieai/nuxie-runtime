use crate::functions::is_constant_literal::is_constant_literal;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_table::{AstExprTable, ItemKind};

#[allow(non_snake_case)]
pub fn is_literal_table(expr: *mut AstExpr) -> bool {
    if expr.is_null() {
        return false;
    }

    let table = unsafe {
        crate::rtti::ast_node_as::<AstExprTable>(expr as *mut crate::records::ast_node::AstNode)
    };
    if table.is_null() {
        return false;
    }

    let items = unsafe { &(*table).items };
    for item in items.iter() {
        match item.kind {
            ItemKind::General => {
                return false;
            }
            ItemKind::Record | ItemKind::List => {
                if !is_constant_literal(item.value) && !is_literal_table(item.value) {
                    return false;
                }
            }
        }
    }

    true
}
