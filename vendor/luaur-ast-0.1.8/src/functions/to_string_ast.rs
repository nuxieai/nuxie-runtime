#[allow(non_snake_case)]
pub fn to_string(op: crate::records::ast_expr_unary::AstExprUnaryOp) -> alloc::string::String {
    match op {
        crate::records::ast_expr_unary::AstExprUnaryOp::Minus => alloc::string::String::from("-"),
        crate::records::ast_expr_unary::AstExprUnaryOp::Not => alloc::string::String::from("not"),
        crate::records::ast_expr_unary::AstExprUnaryOp::Len => alloc::string::String::from("#"),
        _ => {
            luaur_common::LUAU_ASSERT!(false);
            alloc::string::String::new()
        }
    }
}

// Pinned overload name advertised by the dependency cards.
#[allow(unused_imports, non_snake_case)]
pub use to_string as to_string_ast_expr_unary_op;
