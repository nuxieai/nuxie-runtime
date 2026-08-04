use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_call::AstExprCall;
use crate::records::ast_expr_varargs::AstExprVarargs;
use crate::records::ast_node::AstNode;
use crate::records::temp_vector::TempVector;
use crate::rtti::ast_node_is;

pub fn is_enough_values(values: &mut TempVector<'_, *mut AstExpr>, expected: usize) -> bool {
    if values.size() > 0 {
        let last = values.back();
        unsafe {
            let node = *last as *mut AstNode;
            if ast_node_is::<AstExprCall>(&*node) || ast_node_is::<AstExprVarargs>(&*node) {
                return true;
            }
        }
    }
    values.size() == expected
}
