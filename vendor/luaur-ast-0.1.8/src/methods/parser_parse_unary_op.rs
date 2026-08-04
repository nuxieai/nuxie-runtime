use crate::records::ast_expr_unary::AstExprUnaryOp;
use crate::records::lexeme::Lexeme;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub(crate) fn parse_unary_op(&self, l: &Lexeme) -> Option<AstExprUnaryOp> {
        if l.r#type == crate::records::lexeme::Type::ReservedNot {
            Some(AstExprUnaryOp::Not)
        } else if l.r#type == crate::records::lexeme::Type('-' as i32) {
            Some(AstExprUnaryOp::Minus)
        } else if l.r#type == crate::records::lexeme::Type('#' as i32) {
            Some(AstExprUnaryOp::Len)
        } else {
            None
        }
    }
}

pub fn parser_parse_unary_op(this: &Parser, l: &Lexeme) -> Option<AstExprUnaryOp> {
    this.parse_unary_op(l)
}
