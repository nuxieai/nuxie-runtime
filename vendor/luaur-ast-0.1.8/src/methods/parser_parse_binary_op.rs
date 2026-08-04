use crate::records::ast_expr_binary::AstExprBinary_Op as Op;
use crate::records::lexeme::Lexeme;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub(crate) fn parse_binary_op(&self, l: &Lexeme) -> Option<Op> {
        if l.r#type == crate::records::lexeme::Type('+' as i32) {
            Some(Op::Add)
        } else if l.r#type == crate::records::lexeme::Type('-' as i32) {
            Some(Op::Sub)
        } else if l.r#type == crate::records::lexeme::Type('*' as i32) {
            Some(Op::Mul)
        } else if l.r#type == crate::records::lexeme::Type('/' as i32) {
            Some(Op::Div)
        } else if l.r#type == crate::records::lexeme::Type::FloorDiv {
            Some(Op::FloorDiv)
        } else if l.r#type == crate::records::lexeme::Type('%' as i32) {
            Some(Op::Mod)
        } else if l.r#type == crate::records::lexeme::Type('^' as i32) {
            Some(Op::Pow)
        } else if l.r#type == crate::records::lexeme::Type::Dot2 {
            Some(Op::Concat)
        } else if l.r#type == crate::records::lexeme::Type::NotEqual {
            Some(Op::CompareNe)
        } else if l.r#type == crate::records::lexeme::Type::Equal {
            Some(Op::CompareEq)
        } else if l.r#type == crate::records::lexeme::Type('<' as i32) {
            Some(Op::CompareLt)
        } else if l.r#type == crate::records::lexeme::Type::LessEqual {
            Some(Op::CompareLe)
        } else if l.r#type == crate::records::lexeme::Type('>' as i32) {
            Some(Op::CompareGt)
        } else if l.r#type == crate::records::lexeme::Type::GreaterEqual {
            Some(Op::CompareGe)
        } else if l.r#type == crate::records::lexeme::Type::ReservedAnd {
            Some(Op::And)
        } else if l.r#type == crate::records::lexeme::Type::ReservedOr {
            Some(Op::Or)
        } else {
            None
        }
    }
}

#[allow(non_snake_case)]
pub fn parser_parse_binary_op(this: &Parser, l: &Lexeme) -> Option<Op> {
    this.parse_binary_op(l)
}
