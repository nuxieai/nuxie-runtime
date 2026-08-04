use crate::records::ast_expr_binary::AstExprBinary_Op;
use crate::records::lexeme::Lexeme;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub(crate) fn parse_compound_op(&self, l: &Lexeme) -> Option<AstExprBinary_Op> {
        if l.r#type == crate::records::lexeme::Type::AddAssign {
            Some(AstExprBinary_Op::Add)
        } else if l.r#type == crate::records::lexeme::Type::SubAssign {
            Some(AstExprBinary_Op::Sub)
        } else if l.r#type == crate::records::lexeme::Type::MulAssign {
            Some(AstExprBinary_Op::Mul)
        } else if l.r#type == crate::records::lexeme::Type::DivAssign {
            Some(AstExprBinary_Op::Div)
        } else if l.r#type == crate::records::lexeme::Type::FloorDivAssign {
            Some(AstExprBinary_Op::FloorDiv)
        } else if l.r#type == crate::records::lexeme::Type::ModAssign {
            Some(AstExprBinary_Op::Mod)
        } else if l.r#type == crate::records::lexeme::Type::PowAssign {
            Some(AstExprBinary_Op::Pow)
        } else if l.r#type == crate::records::lexeme::Type::ConcatAssign {
            Some(AstExprBinary_Op::Concat)
        } else {
            None
        }
    }
}
