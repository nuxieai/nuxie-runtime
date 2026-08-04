use crate::records::ast_expr_binary::AstExprBinary_Op;
use alloc::string::String;

#[allow(non_snake_case)]
pub fn to_string(op: AstExprBinary_Op) -> String {
    match op {
        AstExprBinary_Op::Add => String::from("+"),
        AstExprBinary_Op::Sub => String::from("-"),
        AstExprBinary_Op::Mul => String::from("*"),
        AstExprBinary_Op::Div => String::from("/"),
        AstExprBinary_Op::FloorDiv => String::from("//"),
        AstExprBinary_Op::Mod => String::from("%"),
        AstExprBinary_Op::Pow => String::from("^"),
        AstExprBinary_Op::Concat => String::from(".."),
        AstExprBinary_Op::CompareNe => String::from("~="),
        AstExprBinary_Op::CompareEq => String::from("=="),
        AstExprBinary_Op::CompareLt => String::from("<"),
        AstExprBinary_Op::CompareLe => String::from("<="),
        AstExprBinary_Op::CompareGt => String::from(">"),
        AstExprBinary_Op::CompareGe => String::from(">="),
        AstExprBinary_Op::And => String::from("and"),
        AstExprBinary_Op::Or => String::from("or"),
        _ => {
            luaur_common::LUAU_ASSERT!(false);
            String::new()
        }
    }
}

// Pinned overload name advertised by the dependency cards.
#[allow(unused_imports, non_snake_case)]
pub use to_string as to_string_ast_expr_binary_op;
