use crate::records::ast_expr_unary::AstExprUnaryOp;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;

impl Parser {
    pub fn check_unary_confusables(&mut self) -> Option<AstExprUnaryOp> {
        let curr = self.lexer.current();

        // early-out: need to check if this is a possible confusable quickly
        if curr.r#type != Type('!' as i32) {
            return None;
        }

        // slow path: possible confusable
        let start = curr.location;

        if curr.r#type == Type('!' as i32) {
            self.report_location_c_char_item(
                start,
                format_args!("Unexpected '!'; did you mean 'not'?"),
            );
            return Some(AstExprUnaryOp::Not);
        }

        None
    }
}

pub fn parser_check_unary_confusables(this: &mut Parser) -> Option<AstExprUnaryOp> {
    this.check_unary_confusables()
}
