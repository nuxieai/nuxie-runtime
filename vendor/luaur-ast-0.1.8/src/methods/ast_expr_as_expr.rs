use crate::records::ast_expr::AstExpr;

impl AstExpr {
    pub fn as_expr(&mut self) -> *mut AstExpr {
        self as *mut AstExpr
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_as_expr(this: *mut AstExpr) -> *mut AstExpr {
    this
}
